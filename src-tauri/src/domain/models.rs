use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::hashing::content_hash;
use crate::error::{AppError, AppResult};

// ---------------------------------------------------------------------------
// Nuvem
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    #[default]
    R2,
    S3,
    Custom,
}

/// Configuração de um bucket compatível com S3 (Cloudflare R2, Amazon S3 ou outro).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CloudConfig {
    pub provider: ProviderKind,
    /// Cloudflare R2: Account ID usado para montar o endpoint.
    pub account_id: String,
    /// Amazon S3 / compatível: região de assinatura.
    pub region: String,
    /// S3 compatível: endpoint completo (https://...).
    pub endpoint: String,
    pub bucket: String,
    /// Pasta raiz dentro do bucket (opcional).
    pub prefix: String,
    pub access_key_id: String,
    // TODO(segurança): mover para o Windows Credential Manager (crate `keyring`).
    pub secret_access_key: String,
}

impl CloudConfig {
    pub fn validate(&self) -> AppResult<()> {
        let mut missing = Vec::new();
        let blank = |s: &str| s.trim().is_empty();

        match self.provider {
            ProviderKind::R2 if blank(&self.account_id) => missing.push("Account ID"),
            ProviderKind::S3 if blank(&self.region) => missing.push("Região"),
            ProviderKind::Custom if blank(&self.endpoint) => missing.push("Endpoint"),
            _ => {}
        }
        if blank(&self.bucket) {
            missing.push("Bucket");
        }
        if blank(&self.access_key_id) {
            missing.push("Access Key ID");
        }
        if blank(&self.secret_access_key) {
            missing.push("Secret Access Key");
        }
        if !missing.is_empty() {
            return Err(AppError::Validation(format!("Preencha: {}.", missing.join(", "))));
        }

        if self.provider == ProviderKind::Custom {
            let ep = self.endpoint.trim();
            if !(ep.starts_with("https://") || ep.starts_with("http://")) {
                return Err(AppError::Validation("O endpoint deve começar com https://".into()));
            }
        }
        Ok(())
    }

    /// Endpoint efetivo. `None` usa o endpoint padrão da AWS para a região.
    pub fn endpoint_url(&self) -> Option<String> {
        match self.provider {
            ProviderKind::R2 => Some(format!(
                "https://{}.r2.cloudflarestorage.com",
                self.account_id.trim()
            )),
            ProviderKind::S3 => None,
            ProviderKind::Custom => Some(self.endpoint.trim().trim_end_matches('/').to_string()),
        }
    }

    pub fn signing_region(&self) -> String {
        match self.provider {
            ProviderKind::R2 => "auto".into(),
            ProviderKind::S3 => self.region.trim().into(),
            ProviderKind::Custom if self.region.trim().is_empty() => "us-east-1".into(),
            ProviderKind::Custom => self.region.trim().into(),
        }
    }

    /// Monta a chave completa de um objeto aplicando o prefixo configurado.
    pub fn key(&self, relative: &str) -> String {
        let prefix = self.prefix.trim().trim_matches('/');
        let relative = relative.trim_start_matches('/');
        if prefix.is_empty() {
            relative.to_string()
        } else {
            format!("{prefix}/{relative}")
        }
    }
}

// ---------------------------------------------------------------------------
// Configurações do app
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Pcsx2Settings {
    pub enabled: bool,
    pub auto_sync: bool,
    /// Pasta de dados do PCSX2. `None` = detectar automaticamente (Documentos\PCSX2).
    pub data_dir: Option<String>,
}

impl Default for Pcsx2Settings {
    fn default() -> Self {
        Self { enabled: true, auto_sync: true, data_dir: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub cloud: Option<CloudConfig>,
    pub pcsx2: Pcsx2Settings,
    pub backups_to_keep: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { cloud: None, pcsx2: Pcsx2Settings::default(), backups_to_keep: 10 }
    }
}

// ---------------------------------------------------------------------------
// Unidades de sincronização
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitKind {
    /// Um único arquivo (ex.: memory card `.ps2`).
    File,
    /// Uma pasta com vários arquivos (ex.: um jogo dentro de um memory card em pasta).
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// Caminho relativo à unidade, com `/` como separador.
    pub path: String,
    pub sha256: String,
    pub size: u64,
    /// Arquivos voláteis são transferidos, mas não entram no hash de conteúdo.
    #[serde(default)]
    pub volatile: bool,
}

/// Menor porção de dados sincronizada de forma atômica.
#[derive(Debug, Clone)]
pub struct SyncUnit {
    /// ID estável e independente de máquina, ex.: `pcsx2/memcards/Mcd001`.
    pub id: String,
    pub source_id: String,
    /// Agrupamento visual, ex.: nome do memory card.
    pub group: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    pub kind: UnitKind,
    pub local_path: PathBuf,
    pub files: Vec<FileEntry>,
    pub exists_locally: bool,
    pub modified_at: Option<DateTime<Utc>>,
    /// Algum arquivo está em uso por outro processo e não pôde ser lido agora.
    pub busy: bool,
}

/// Mensagem exibida para saves cujos arquivos estão em uso pelo emulador.
pub const BUSY_MESSAGE: &str = "Em uso pelo emulador. Será sincronizado quando o arquivo for liberado.";

impl SyncUnit {
    pub fn content_hash(&self) -> Option<String> {
        self.exists_locally.then(|| content_hash(&self.files))
    }

    pub fn size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
}

/// Descrição de uma unidade na nuvem. É gravado por último em cada upload
/// (commit atômico) e aponta para blobs endereçados por conteúdo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteManifest {
    pub version: u32,
    pub unit_id: String,
    pub kind: UnitKind,
    pub title: String,
    pub hash: String,
    pub files: Vec<FileEntry>,
    pub updated_at: DateTime<Utc>,
    pub machine: String,
    /// Lápide: o save foi excluído e as outras máquinas devem apagar suas cópias.
    #[serde(default)]
    pub deleted: bool,
}

impl RemoteManifest {
    pub fn tombstone(unit: &SyncUnit, title: String, machine: String) -> Self {
        Self {
            version: 1,
            unit_id: unit.id.clone(),
            kind: unit.kind,
            title,
            hash: String::new(),
            files: Vec::new(),
            updated_at: Utc::now(),
            machine,
            deleted: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncAction {
    Nothing,
    InSync,
    Upload,
    Download,
    /// O save foi excluído na nuvem e a cópia local não mudou: apagar aqui também.
    DeleteLocal,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Resolution {
    KeepLocal,
    KeepCloud,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnitStatus {
    Synced,
    Pending,
    Conflict,
    /// Download adiado porque o emulador está aberto.
    Deferred,
    Error,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UnitSyncState {
    pub base_hash: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub status: UnitStatus,
    pub message: Option<String>,
    pub remote_machine: Option<String>,
    /// Conflito causado por exclusão em outra máquina (a nuvem tem uma lápide).
    pub remote_deleted: bool,
    // Metadados para exibir unidades que ainda não existem localmente.
    pub title: String,
    pub group: String,
    pub kind: Option<UnitKind>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SyncStateFile {
    pub units: HashMap<String, UnitSyncState>,
}

// ---------------------------------------------------------------------------
// Atividade e eventos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActivityKind {
    Upload,
    Download,
    Delete,
    Conflict,
    Deferred,
    Error,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub id: String,
    pub at: DateTime<Utc>,
    pub source_id: String,
    pub unit_id: Option<String>,
    pub title: String,
    pub kind: ActivityKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncTrigger {
    Manual,
    Watcher,
    Startup,
    Retry,
    Resolution,
    Deletion,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSummary {
    pub source_id: String,
    pub uploaded: u32,
    pub downloaded: u32,
    pub deleted: u32,
    pub in_sync: u32,
    pub conflicts: u32,
    pub deferred: u32,
    pub errors: u32,
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnitPhase {
    Checking,
    Uploading,
    Downloading,
    Deleting,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum SyncEvent {
    Started { source_id: String, trigger: SyncTrigger, total: usize },
    Unit {
        source_id: String,
        unit_id: String,
        title: String,
        phase: UnitPhase,
        status: Option<UnitStatus>,
        index: usize,
        total: usize,
    },
    Finished { summary: SyncSummary },
    /// Arquivos locais da fonte mudaram (watcher); a interface deve recarregar a biblioteca.
    LocalChanged { source_id: String },
}

// ---------------------------------------------------------------------------
// Fontes (emuladores)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub id: String,
    pub name: String,
    pub detected: bool,
    pub data_dir: Option<String>,
    pub saves_dir: Option<String>,
    pub config_found: bool,
    /// Instalação portátil (dados ao lado do executável).
    pub portable: bool,
    /// Memory cards encontrados (arquivos e pastas).
    pub cards: Vec<String>,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r2_endpoint_and_region() {
        let cfg = CloudConfig {
            provider: ProviderKind::R2,
            account_id: " abc123 ".into(),
            ..Default::default()
        };
        assert_eq!(cfg.endpoint_url().unwrap(), "https://abc123.r2.cloudflarestorage.com");
        assert_eq!(cfg.signing_region(), "auto");
    }

    #[test]
    fn key_applies_prefix() {
        let mut cfg = CloudConfig::default();
        assert_eq!(cfg.key("a/b"), "a/b");
        cfg.prefix = "/saves/".into();
        assert_eq!(cfg.key("a/b"), "saves/a/b");
    }

    #[test]
    fn validation_lists_missing_fields() {
        let cfg = CloudConfig { provider: ProviderKind::S3, ..Default::default() };
        let err = cfg.validate().unwrap_err().to_string();
        assert!(err.contains("Região") && err.contains("Bucket"));
    }
}
