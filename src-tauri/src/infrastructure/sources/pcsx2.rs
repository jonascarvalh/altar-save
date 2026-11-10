//! Fonte PCSX2: memory cards em arquivo (`.ps2`) e em pasta (folder memory cards).

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{DateTime, Utc};

use super::{ini::Ini, ps2_icon_sys};
use crate::{
    domain::{
        hashing::sha256_hex,
        models::{FileEntry, Pcsx2Settings, SourceInfo, SyncUnit, UnitKind},
        ports::{RemoteScope, SaveSource, SourceScan},
    },
    error::{AppResult, is_file_in_use},
    infrastructure::fs_store::TEMP_MARKER,
};

pub const PCSX2_ID: &str = "pcsx2";
const PROCESS_NAMES: &[&str] = &["pcsx2"];
const SUPERBLOCK: &str = "_pcsx2_superblock";
const INDEX: &str = "_pcsx2_index";
const FILE_CARD_DEBOUNCE: Duration = Duration::from_secs(5);
/// Memory cards em pasta são gravados com atraso pelo PCSX2.
const FOLDER_CARD_DEBOUNCE: Duration = Duration::from_secs(10);

/// Pastas e slots resolvidos a partir do `PCSX2.ini`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcsx2Layout {
    pub data_dir: PathBuf,
    pub cards_dir: PathBuf,
    /// Pasta onde o PCSX2 guarda as capas (`<serial>.png`, etc.).
    pub covers_dir: PathBuf,
    pub config_found: bool,
    /// (número do slot, nome do arquivo/pasta do cartão)
    pub slots: Vec<(u8, String)>,
}

pub fn parse_layout(data_dir: &Path, ini_text: Option<&str>) -> Pcsx2Layout {
    let ini = ini_text.map(Ini::parse);
    let get = |section: &str, key: &str| ini.as_ref().and_then(|i| i.get(section, key)).filter(|v| !v.is_empty());
    let resolve = |value: &str| {
        let path = PathBuf::from(value);
        if path.is_absolute() { path } else { data_dir.join(path) }
    };

    let cards_dir = resolve(get("Folders", "MemoryCards").unwrap_or("memcards"));
    let covers_dir = resolve(get("Folders", "Covers").unwrap_or("covers"));

    let slots = (1..=2u8)
        .filter_map(|n| {
            let enabled = get("MemoryCards", &format!("Slot{n}_Enable"))
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(true);
            let file = get("MemoryCards", &format!("Slot{n}_Filename"))?;
            enabled.then(|| (n, file.to_string()))
        })
        .collect();

    Pcsx2Layout { data_dir: data_dir.to_path_buf(), cards_dir, covers_dir, config_found: ini.is_some(), slots }
}

/// Serial do disco a partir do nome da pasta de um save de PS2.
///
/// Formato: 2 caracteres de região (`BA` América, `BE` Europa, `BI` Japão…), 10 do serial
/// (`SCUS-97399`) e um identificador livre do jogo. Ex.: `BASCUS-97399GodOfWar` → `SCUS-97399`.
pub fn serial_from_save_dir(name: &str) -> Option<String> {
    if !name.starts_with('B') {
        return None;
    }
    let serial = name.get(2..12)?;
    let (prefix, number) = serial.split_once('-')?;
    let valid = prefix.len() == 4
        && prefix.chars().all(|c| c.is_ascii_alphabetic())
        && number.len() == 5
        && number.chars().all(|c| c.is_ascii_digit());
    valid.then(|| serial.to_ascii_uppercase())
}

const COVER_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

/// Procura a capa do jogo do jeito que o PCSX2 nomeia: `<serial>.<ext>`.
fn find_cover(covers_dir: &Path, serial: &str) -> Option<PathBuf> {
    COVER_EXTENSIONS
        .iter()
        .map(|ext| covers_dir.join(format!("{serial}.{ext}")))
        .find(|path| path.is_file())
}

/// A pasta parece ser a pasta de dados do PCSX2 (instalada ou portátil)?
fn is_data_dir(path: &Path) -> bool {
    path.join("inis").is_dir() || is_portable_install(path)
}

fn is_portable_install(path: &Path) -> bool {
    path.join("portable.txt").is_file() || path.join("portable.ini").is_file()
}

/// Aceita a pasta escolhida em qualquer nível: a pasta do PCSX2, a pasta `memcards`
/// (ou outra subpasta) ou o próprio `pcsx2-qt.exe`.
pub fn normalize_data_dir(path: &Path) -> PathBuf {
    let path = if path.is_file() { path.parent().unwrap_or(path) } else { path };
    if is_data_dir(path) {
        return path.to_path_buf();
    }
    match path.parent() {
        Some(parent) if is_data_dir(parent) => parent.to_path_buf(),
        _ => path.to_path_buf(),
    }
}

/// Possíveis pastas de dados: instalações portáteis do PCSX2 em execução e a pasta padrão
/// (Documentos\PCSX2), sem duplicatas.
pub fn candidate_data_dirs(running_executables: &[PathBuf]) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = running_executables
        .iter()
        .filter_map(|exe| exe.parent())
        .filter(|dir| is_portable_install(dir))
        .map(Path::to_path_buf)
        .collect();
    candidates.extend(default_data_dir().filter(|d| d.is_dir()));

    let mut seen = std::collections::HashSet::new();
    candidates.retain(|p| seen.insert(p.to_string_lossy().to_lowercase()));
    candidates
}

pub fn process_names() -> &'static [&'static str] {
    PROCESS_NAMES
}

pub struct Pcsx2Source {
    data_dir: Option<PathBuf>,
}

impl Pcsx2Source {
    pub fn new(settings: &Pcsx2Settings) -> Self {
        let data_dir = settings
            .data_dir
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
            .map(|d| normalize_data_dir(Path::new(d)))
            .or_else(default_data_dir);
        Self { data_dir }
    }

    /// Nomes dos memory cards (arquivo ou pasta) presentes na pasta de cartões.
    fn card_names(cards_dir: &Path) -> Vec<String> {
        sorted_entries(cards_dir)
            .unwrap_or_default()
            .into_iter()
            .filter(|e| {
                let path = e.path();
                let name = e.file_name().to_string_lossy().to_string();
                !name.starts_with('.')
                    && ((path.is_file() && has_ps2_extension(&name)) || path.join(SUPERBLOCK).is_file())
            })
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect()
    }

    #[cfg(test)]
    pub fn with_data_dir(data_dir: impl Into<PathBuf>) -> Self {
        Self { data_dir: Some(data_dir.into()) }
    }

    fn layout(&self) -> Option<Pcsx2Layout> {
        let data_dir = self.data_dir.as_ref()?;
        let ini = fs::read_to_string(data_dir.join("inis").join("PCSX2.ini")).ok();
        Some(parse_layout(data_dir, ini.as_deref()))
    }

    fn slot_tags(layout: &Pcsx2Layout, card_name: &str) -> Vec<String> {
        layout
            .slots
            .iter()
            .filter(|(_, file)| file.eq_ignore_ascii_case(card_name))
            .map(|(n, _)| format!("Slot {n}"))
            .collect()
    }

    fn file_card_unit(&self, layout: &Pcsx2Layout, path: &Path, name: &str) -> AppResult<SyncUnit> {
        let stem = card_stem(name);
        let bytes = read_unless_in_use(path)?;
        let files = bytes
            .as_ref()
            .map(|bytes| FileEntry {
                path: name.to_string(),
                sha256: sha256_hex(bytes),
                size: bytes.len() as u64,
                volatile: false,
            })
            .into_iter()
            .collect();
        Ok(SyncUnit {
            id: format!("{PCSX2_ID}/memcards/{stem}"),
            source_id: PCSX2_ID.into(),
            group: stem.clone(),
            title: stem,
            subtitle: Some(name.to_string()),
            tags: Self::slot_tags(layout, name),
            kind: UnitKind::File,
            local_path: path.to_path_buf(),
            files,
            exists_locally: true,
            modified_at: modified_at(path),
            busy: bytes.is_none(),
        })
    }

    fn folder_save_unit(&self, card_stem: &str, tags: &[String], dir: &Path, name: &str) -> AppResult<SyncUnit> {
        let mut files = Vec::new();
        let busy = collect_files(dir, dir, &mut files)?;
        let modified = files
            .iter()
            .filter(|(entry, _)| !entry.volatile)
            .filter_map(|(_, m)| *m)
            .max();

        Ok(SyncUnit {
            id: format!("{PCSX2_ID}/folders/{card_stem}/{name}"),
            source_id: PCSX2_ID.into(),
            group: card_stem.into(),
            title: ps2_icon_sys::read_title(dir).unwrap_or_else(|| name.into()),
            subtitle: Some(name.into()),
            tags: tags.to_vec(),
            kind: UnitKind::Directory,
            local_path: dir.to_path_buf(),
            files: files.into_iter().map(|(entry, _)| entry).collect(),
            exists_locally: true,
            modified_at: modified,
            busy,
        })
    }
}

/// Lê o arquivo; `None` se ele estiver aberto com exclusividade por outro processo (ex.: PCSX2 jogando).
fn read_unless_in_use(path: &Path) -> AppResult<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(err) if is_file_in_use(&err) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

impl SaveSource for Pcsx2Source {
    fn id(&self) -> &str {
        PCSX2_ID
    }

    fn info(&self) -> SourceInfo {
        let layout = self.layout();
        let message = match &layout {
            None => Some("Não foi possível localizar a pasta Documentos.".to_string()),
            Some(l) if !l.data_dir.is_dir() => Some(format!("Pasta do PCSX2 não encontrada: {}", l.data_dir.display())),
            Some(l) if !l.cards_dir.is_dir() => {
                Some(format!("Pasta de memory cards não encontrada: {}", l.cards_dir.display()))
            }
            Some(l) if !l.config_found => Some("PCSX2.ini não encontrado; usando a pasta padrão 'memcards'.".into()),
            _ => None,
        };
        SourceInfo {
            id: PCSX2_ID.into(),
            name: "PCSX2".into(),
            detected: layout.as_ref().is_some_and(|l| l.cards_dir.is_dir()),
            data_dir: layout.as_ref().map(|l| l.data_dir.display().to_string()),
            saves_dir: layout.as_ref().map(|l| l.cards_dir.display().to_string()),
            config_found: layout.as_ref().is_some_and(|l| l.config_found),
            portable: layout.as_ref().is_some_and(|l| is_portable_install(&l.data_dir)),
            cards: layout.as_ref().map(|l| Self::card_names(&l.cards_dir)).unwrap_or_default(),
            message,
        }
    }

    fn scan(&self) -> AppResult<SourceScan> {
        let Some(layout) = self.layout() else {
            return Ok(SourceScan::default());
        };
        if !layout.cards_dir.is_dir() {
            return Ok(SourceScan::default());
        }

        let mut scan = SourceScan::default();
        // Cartões em arquivo que existem só na nuvem são baixados para a pasta de memcards.
        scan.remote_scopes.push(RemoteScope {
            id_prefix: format!("{PCSX2_ID}/memcards/"),
            kind: UnitKind::File,
            group: None,
            local_parent: layout.cards_dir.clone(),
            local_suffix: ".ps2".into(),
        });

        for entry in sorted_entries(&layout.cards_dir)? {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.contains(TEMP_MARKER) {
                continue;
            }
            let file_type = entry.file_type()?;

            if file_type.is_file() && has_ps2_extension(&name) {
                scan.units.push(self.file_card_unit(&layout, &path, &name)?);
            } else if file_type.is_dir() && path.join(SUPERBLOCK).is_file() {
                let stem = card_stem(&name);
                let tags = Self::slot_tags(&layout, &name);
                scan.remote_scopes.push(RemoteScope {
                    id_prefix: format!("{PCSX2_ID}/folders/{stem}/"),
                    kind: UnitKind::Directory,
                    group: Some(stem.clone()),
                    local_parent: path.clone(),
                    local_suffix: String::new(),
                });
                for save in sorted_entries(&path)? {
                    let save_name = save.file_name().to_string_lossy().to_string();
                    if save.file_type()?.is_dir() && !save_name.starts_with('.') && !save_name.contains(TEMP_MARKER) {
                        scan.units.push(self.folder_save_unit(&stem, &tags, &save.path(), &save_name)?);
                    }
                }
            }
        }
        Ok(scan)
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        self.layout().map(|l| l.cards_dir).filter(|d| d.is_dir()).into_iter().collect()
    }

    fn watch_debounce(&self) -> Duration {
        let has_folder_cards = self
            .layout()
            .and_then(|l| fs::read_dir(l.cards_dir).ok())
            .is_some_and(|mut entries| entries.any(|e| e.is_ok_and(|e| e.path().join(SUPERBLOCK).is_file())));
        if has_folder_cards { FOLDER_CARD_DEBOUNCE } else { FILE_CARD_DEBOUNCE }
    }

    fn process_names(&self) -> &'static [&'static str] {
        process_names()
    }

    fn find_covers(&self, units: &[SyncUnit]) -> HashMap<String, PathBuf> {
        let Some(layout) = self.layout().filter(|l| l.covers_dir.is_dir()) else {
            return HashMap::new();
        };
        units
            .iter()
            .filter(|u| u.kind == UnitKind::Directory)
            .filter_map(|u| {
                let dir_name = u.local_path.file_name()?.to_str()?;
                let cover = find_cover(&layout.covers_dir, &serial_from_save_dir(dir_name)?)?;
                Some((u.id.clone(), cover))
            })
            .collect()
    }
}

fn default_data_dir() -> Option<PathBuf> {
    dirs::document_dir().map(|d| d.join("PCSX2"))
}

fn has_ps2_extension(name: &str) -> bool {
    Path::new(name).extension().is_some_and(|e| e.eq_ignore_ascii_case("ps2"))
}

/// `Mcd001.ps2` → `Mcd001` (cartões em pasta também costumam ter a extensão no nome).
fn card_stem(name: &str) -> String {
    if has_ps2_extension(name) { name[..name.len() - 4].to_string() } else { name.to_string() }
}

fn sorted_entries(dir: &Path) -> AppResult<Vec<fs::DirEntry>> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.file_name());
    Ok(entries)
}

fn modified_at(path: &Path) -> Option<DateTime<Utc>> {
    fs::metadata(path).and_then(|m| m.modified()).ok().map(DateTime::<Utc>::from)
}

/// Coleta os arquivos da pasta. Retorna `true` se algum estava em uso e não pôde ser lido.
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(FileEntry, Option<DateTime<Utc>>)>) -> AppResult<bool> {
    let mut busy = false;
    for entry in sorted_entries(dir)? {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            busy |= collect_files(root, &path, out)?;
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let Some(bytes) = read_unless_in_use(&path)? else {
            busy = true;
            continue;
        };
        let file_name = entry.file_name();
        out.push((
            FileEntry {
                path: relative,
                sha256: sha256_hex(&bytes),
                size: bytes.len() as u64,
                volatile: file_name == INDEX,
            },
            modified_at(&path),
        ));
    }
    Ok(busy)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pasta absoluta válida no SO atual (os testes rodam no Windows e no Linux).
    fn abs(parts: &[&str]) -> PathBuf {
        parts.iter().fold(std::env::temp_dir(), |p, part| p.join(part))
    }

    #[test]
    fn default_layout_without_ini() {
        let root = abs(&["Documents", "PCSX2"]);
        let layout = parse_layout(&root, None);
        assert_eq!(layout.cards_dir, root.join("memcards"));
        assert!(!layout.config_found);
        assert!(layout.slots.is_empty());
    }

    #[test]
    fn layout_reads_folders_and_slots() {
        let ini = "[Folders]\nMemoryCards = cards\n[MemoryCards]\nSlot1_Enable = true\nSlot1_Filename = Mcd001.ps2\nSlot2_Enable = false\nSlot2_Filename = Mcd002.ps2\n";
        let root = abs(&["PCSX2"]);
        let layout = parse_layout(&root, Some(ini));
        assert_eq!(layout.cards_dir, root.join("cards"));
        assert_eq!(layout.slots, vec![(1, "Mcd001.ps2".to_string())]);
    }

    #[test]
    fn layout_accepts_absolute_cards_dir() {
        let cards = abs(&["Saves", "PS2"]);
        let ini = format!("[Folders]\nMemoryCards = {}\n", cards.display());
        let layout = parse_layout(&abs(&["PCSX2"]), Some(&ini));
        assert_eq!(layout.cards_dir, cards);
    }

    /// Reproduz uma instalação portátil (dados ao lado do pcsx2-qt.exe) com um cartão convertido para pasta.
    fn portable_install(root: &Path) -> PathBuf {
        let install = root.join("Games").join("PCSX2");
        fs::create_dir_all(install.join("inis")).unwrap();
        fs::write(install.join("portable.txt"), b"").unwrap();
        fs::write(install.join("pcsx2-qt.exe"), b"").unwrap();
        fs::write(
            install.join("inis").join("PCSX2.ini"),
            "[Folders]\nMemoryCards = memcards\n[MemoryCards]\nSlot1_Enable = true\nSlot1_Filename = Mcd001_converted.ps2\nSlot2_Enable = true\nSlot2_Filename = Mcd002.ps2\n",
        )
        .unwrap();
        let cards = install.join("memcards");
        let folder = cards.join("Mcd001_converted.ps2");
        fs::create_dir_all(folder.join("BASCUS-97399GodOfWar")).unwrap();
        fs::write(folder.join(SUPERBLOCK), b"sb").unwrap();
        fs::write(folder.join("BASCUS-97399GodOfWar").join("save.dat"), b"kratos").unwrap();
        fs::write(cards.join("Mcd002.ps2"), b"card").unwrap();
        install
    }

    #[test]
    fn extracts_serial_from_save_dir_names() {
        assert_eq!(serial_from_save_dir("BASCUS-97399GodOfWar").as_deref(), Some("SCUS-97399"));
        assert_eq!(serial_from_save_dir("BESLES-82034KH2").as_deref(), Some("SLES-82034"));
        assert_eq!(serial_from_save_dir("BISLPM-66233").as_deref(), Some("SLPM-66233"));
        assert_eq!(serial_from_save_dir("BAslus-21005x").as_deref(), Some("SLUS-21005"));
        assert_eq!(serial_from_save_dir("GAME1"), None);
        assert_eq!(serial_from_save_dir("BASCUS97399XX"), None);
        assert_eq!(serial_from_save_dir("BAçãoS-97399"), None);
    }

    #[test]
    fn finds_cover_by_serial_in_configured_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let install = portable_install(tmp.path());
        fs::create_dir_all(install.join("covers")).unwrap();
        fs::write(install.join("covers").join("SCUS-97399.png"), b"png").unwrap();

        let source = Pcsx2Source::with_data_dir(&install);
        let units = source.scan().unwrap().units;
        let covers = source.find_covers(&units);

        assert_eq!(
            covers.get("pcsx2/folders/Mcd001_converted/BASCUS-97399GodOfWar"),
            Some(&install.join("covers").join("SCUS-97399.png"))
        );
        // Cartão .ps2 inteiro mistura jogos: sem capa.
        assert!(!covers.contains_key("pcsx2/memcards/Mcd002"));
    }

    #[test]
    fn normalizes_any_level_of_portable_install() {
        let tmp = tempfile::tempdir().unwrap();
        let install = portable_install(tmp.path());
        assert_eq!(normalize_data_dir(&install), install);
        assert_eq!(normalize_data_dir(&install.join("memcards")), install);
        assert_eq!(normalize_data_dir(&install.join("pcsx2-qt.exe")), install);
    }

    #[test]
    fn finds_portable_install_from_running_executable() {
        let tmp = tempfile::tempdir().unwrap();
        let install = portable_install(tmp.path());
        let candidates = candidate_data_dirs(&[install.join("pcsx2-qt.exe")]);
        assert_eq!(candidates.first(), Some(&install));
    }

    #[test]
    fn scans_portable_install_with_converted_folder_card() {
        let tmp = tempfile::tempdir().unwrap();
        let install = portable_install(tmp.path());
        let settings = Pcsx2Settings { data_dir: Some(install.join("memcards").display().to_string()), ..Default::default() };
        let source = Pcsx2Source::new(&settings);

        let info = source.info();
        assert!(info.detected && info.portable && info.config_found);
        assert_eq!(info.cards, vec!["Mcd001_converted.ps2", "Mcd002.ps2"]);

        let scan = source.scan().unwrap();
        let summary: Vec<_> = scan.units.iter().map(|u| (u.id.as_str(), u.tags.clone())).collect();
        assert_eq!(
            summary,
            vec![
                ("pcsx2/folders/Mcd001_converted/BASCUS-97399GodOfWar", vec!["Slot 1".to_string()]),
                ("pcsx2/memcards/Mcd002", vec!["Slot 2".to_string()]),
            ]
        );
    }

    #[test]
    fn scans_file_and_folder_cards() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path();
        fs::create_dir_all(data.join("inis")).unwrap();
        fs::write(
            data.join("inis").join("PCSX2.ini"),
            "[MemoryCards]\nSlot1_Filename = Mcd001.ps2\nSlot2_Filename = Mcd002.ps2\n",
        )
        .unwrap();
        let cards = data.join("memcards");
        fs::create_dir_all(&cards).unwrap();
        fs::write(cards.join("Mcd001.ps2"), b"card").unwrap();
        fs::write(cards.join(".Mcd001.ps2.save-nuvem-tmp"), b"tmp").unwrap();

        let folder = cards.join("Mcd002.ps2");
        fs::create_dir_all(folder.join("BASLUS-20778")).unwrap();
        fs::write(folder.join(SUPERBLOCK), b"sb").unwrap();
        fs::write(folder.join("BASLUS-20778").join("save.dat"), b"save").unwrap();
        fs::write(folder.join("BASLUS-20778").join(INDEX), b"idx").unwrap();

        let scan = Pcsx2Source::with_data_dir(data).scan().unwrap();
        let ids: Vec<_> = scan.units.iter().map(|u| u.id.as_str()).collect();
        assert_eq!(ids, vec!["pcsx2/memcards/Mcd001", "pcsx2/folders/Mcd002/BASLUS-20778"]);

        let file_card = &scan.units[0];
        assert_eq!(file_card.tags, vec!["Slot 1"]);
        assert_eq!(file_card.kind, UnitKind::File);

        let save = &scan.units[1];
        assert_eq!(save.tags, vec!["Slot 2"]);
        assert_eq!(save.files.len(), 2);
        assert!(save.files.iter().any(|f| f.path == INDEX && f.volatile));
        assert!(scan.remote_scopes.iter().any(|s| s.id_prefix == "pcsx2/folders/Mcd002/"));
    }
}
