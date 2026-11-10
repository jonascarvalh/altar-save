//! Testes de fluxo: duas "máquinas" compartilhando o mesmo bucket em memória.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    application::sync_service::{SyncDeps, SyncService},
    domain::{
        models::*,
        ports::{ProcessMonitor, SyncStateRepository},
    },
    infrastructure::{
        backups::FsBackupStore,
        events::NoopEventSink,
        fs_store::FsUnitStore,
        persistence::{JsonActivityLog, JsonFile},
        sources::pcsx2::Pcsx2Source,
        storage::memory_storage::MemoryStorage,
    },
};

#[derive(Default)]
struct FakeProcess(AtomicBool);

impl ProcessMonitor for FakeProcess {
    fn is_any_running(&self, _names: &[&str]) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

struct Machine {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    sync: SyncService,
    source: Pcsx2Source,
    state: Arc<JsonFile<SyncStateFile>>,
    process: Arc<FakeProcess>,
}

impl Machine {
    fn new(name: &str, storage: &MemoryStorage) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let pcsx2 = root.join("PCSX2");
        fs::create_dir_all(pcsx2.join("inis")).unwrap();
        fs::create_dir_all(pcsx2.join("memcards")).unwrap();
        fs::write(pcsx2.join("inis").join("PCSX2.ini"), "[Folders]\nMemoryCards = memcards\n").unwrap();

        let settings = Arc::new(JsonFile::<AppSettings>::new(root.join("config.json")));
        settings
            .write(&AppSettings {
                cloud: Some(CloudConfig {
                    provider: ProviderKind::R2,
                    account_id: "acc".into(),
                    bucket: "saves".into(),
                    prefix: "test".into(),
                    access_key_id: "key".into(),
                    secret_access_key: "secret".into(),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .unwrap();
        let state = Arc::new(JsonFile::<SyncStateFile>::new(root.join("sync-state.json")));
        let process = Arc::new(FakeProcess::default());

        let sync = SyncService::new(SyncDeps {
            storage_factory: Arc::new(storage.clone()),
            settings,
            state: state.clone(),
            activity: Arc::new(JsonActivityLog::new(root.join("activity.json"), 100)),
            local: Arc::new(FsUnitStore),
            backups: Arc::new(FsBackupStore::new(root.join("backups"))),
            process: process.clone(),
            events: Arc::new(NoopEventSink),
            machine: name.into(),
        });

        Self { source: Pcsx2Source::with_data_dir(&pcsx2), _tmp: tmp, root, sync, state, process }
    }

    fn cards(&self) -> PathBuf {
        self.root.join("PCSX2").join("memcards")
    }

    async fn sync(&self) -> SyncSummary {
        self.sync.sync_source(&self.source, SyncTrigger::Manual).await.unwrap()
    }

    fn status(&self, unit_id: &str) -> UnitStatus {
        self.state.load().unwrap().units.get(unit_id).map(|u| u.status).unwrap_or_default()
    }

    fn make_folder_card(&self, card: &str) -> PathBuf {
        let dir = self.cards().join(card);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("_pcsx2_superblock"), b"superblock").unwrap();
        dir
    }
}

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

const CARD: &str = "pcsx2/memcards/Mcd001";

#[tokio::test]
async fn file_card_roundtrip_between_machines() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);
    let b = Machine::new("PC-B", &bucket);

    write(&a.cards().join("Mcd001.ps2"), b"save v1");
    let s = a.sync().await;
    assert_eq!(s.uploaded, 1);
    assert!(bucket.keys().iter().any(|k| k == "test/pcsx2/memcards/Mcd001/manifest.json"));

    // B não tem o cartão: baixa da nuvem.
    let s = b.sync().await;
    assert_eq!(s.downloaded, 1);
    assert_eq!(fs::read(b.cards().join("Mcd001.ps2")).unwrap(), b"save v1");

    // Sem mudanças: tudo em dia.
    assert_eq!(a.sync().await.in_sync, 1);
    assert_eq!(b.sync().await.in_sync, 1);

    // A altera → envia; B baixa e faz backup da versão antiga.
    write(&a.cards().join("Mcd001.ps2"), b"save v2");
    assert_eq!(a.sync().await.uploaded, 1);
    assert_eq!(b.sync().await.downloaded, 1);
    assert_eq!(fs::read(b.cards().join("Mcd001.ps2")).unwrap(), b"save v2");
    assert!(b.root.join("backups").join("pcsx2").join("memcards").join("Mcd001").is_dir());

    // Blobs antigos são removidos: apenas 1 blob + manifest.
    let card_keys: Vec<_> = bucket.keys().into_iter().filter(|k| k.contains("Mcd001")).collect();
    assert_eq!(card_keys.len(), 2, "{card_keys:?}");
}

#[tokio::test]
async fn conflict_is_detected_and_resolved() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);
    let b = Machine::new("PC-B", &bucket);

    write(&a.cards().join("Mcd001.ps2"), b"base");
    a.sync().await;
    b.sync().await;

    write(&a.cards().join("Mcd001.ps2"), b"from A");
    write(&b.cards().join("Mcd001.ps2"), b"from B");
    a.sync().await;

    let s = b.sync().await;
    assert_eq!(s.conflicts, 1);
    assert_eq!(b.status(CARD), UnitStatus::Conflict);
    // Nada foi sobrescrito.
    assert_eq!(fs::read(b.cards().join("Mcd001.ps2")).unwrap(), b"from B");

    b.sync.resolve_conflict(&b.source, CARD, Resolution::KeepLocal).await.unwrap();
    assert_eq!(b.status(CARD), UnitStatus::Synced);

    assert_eq!(a.sync().await.downloaded, 1);
    assert_eq!(fs::read(a.cards().join("Mcd001.ps2")).unwrap(), b"from B");
}

#[tokio::test]
async fn keep_cloud_resolution_downloads() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);
    let b = Machine::new("PC-B", &bucket);

    write(&a.cards().join("Mcd001.ps2"), b"from A");
    write(&b.cards().join("Mcd001.ps2"), b"from B");
    a.sync().await;
    assert_eq!(b.sync().await.conflicts, 1);

    b.sync.resolve_conflict(&b.source, CARD, Resolution::KeepCloud).await.unwrap();
    assert_eq!(fs::read(b.cards().join("Mcd001.ps2")).unwrap(), b"from A");
}

#[tokio::test]
async fn download_is_deferred_while_emulator_runs() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);
    let b = Machine::new("PC-B", &bucket);

    write(&a.cards().join("Mcd001.ps2"), b"v1");
    a.sync().await;

    b.process.0.store(true, Ordering::SeqCst);
    assert_eq!(b.sync().await.deferred, 1);
    assert!(!b.cards().join("Mcd001.ps2").exists());
    assert!(!b.sync.should_retry_deferred(&b.source));

    b.process.0.store(false, Ordering::SeqCst);
    assert!(b.sync.should_retry_deferred(&b.source));
    assert_eq!(b.sync().await.downloaded, 1);
}

#[tokio::test]
async fn folder_cards_sync_per_game_without_conflicts() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);
    let b = Machine::new("PC-B", &bucket);

    let card_a = a.make_folder_card("Mcd001.ps2");
    let card_b = b.make_folder_card("Mcd001.ps2");
    write(&card_a.join("GAME1").join("save.dat"), b"g1 v1");
    write(&card_a.join("GAME1").join("_pcsx2_index"), b"idx");
    write(&card_a.join("GAME2").join("save.dat"), b"g2 v1");

    assert_eq!(a.sync().await.uploaded, 2);
    // B descobre os dois jogos pela nuvem.
    assert_eq!(b.sync().await.downloaded, 2);
    assert_eq!(fs::read(card_b.join("GAME1").join("save.dat")).unwrap(), b"g1 v1");
    assert_eq!(fs::read(card_b.join("GAME1").join("_pcsx2_index")).unwrap(), b"idx");

    // Só o índice muda: não gera envio.
    write(&card_a.join("GAME1").join("_pcsx2_index"), b"idx changed");
    let s = a.sync().await;
    assert_eq!((s.uploaded, s.in_sync), (0, 2));

    // Cada máquina altera um jogo diferente: sem conflito.
    write(&card_a.join("GAME1").join("save.dat"), b"g1 v2 (A)");
    write(&card_b.join("GAME2").join("save.dat"), b"g2 v2 (B)");
    assert_eq!(a.sync().await.uploaded, 1);
    let s = b.sync().await;
    assert_eq!((s.uploaded, s.downloaded, s.conflicts), (1, 1, 0));
    let s = a.sync().await;
    assert_eq!((s.downloaded, s.conflicts), (1, 0));

    assert_eq!(fs::read(card_b.join("GAME1").join("save.dat")).unwrap(), b"g1 v2 (A)");
    assert_eq!(fs::read(card_a.join("GAME2").join("save.dat")).unwrap(), b"g2 v2 (B)");
}

#[tokio::test]
async fn card_deleted_locally_and_in_cloud_is_forgotten() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);

    write(&a.cards().join("Mcd001.ps2"), b"card 1");
    write(&a.cards().join("Mcd002.ps2"), b"card 2");
    a.sync().await;
    assert_eq!(a.status("pcsx2/memcards/Mcd002"), UnitStatus::Synced);

    // O usuário apaga o cartão 2 do PC e do bucket.
    fs::remove_file(a.cards().join("Mcd002.ps2")).unwrap();
    bucket.objects.lock().unwrap().retain(|k, _| !k.contains("Mcd002"));

    let s = a.sync().await;
    assert_eq!(s.in_sync, 1);
    let state = a.state.load().unwrap();
    assert!(!state.units.contains_key("pcsx2/memcards/Mcd002"), "estado órfão: {:?}", state.units.keys());
    assert!(state.units.contains_key(CARD));
}

/// Abre o arquivo sem compartilhamento, como o PCSX2 faz enquanto o jogo roda.
#[cfg(windows)]
fn lock_exclusively(path: &Path) -> fs::File {
    use std::os::windows::fs::OpenOptionsExt;
    fs::OpenOptions::new().read(true).write(true).share_mode(0).open(path).unwrap()
}

#[cfg(windows)]
#[tokio::test]
async fn files_in_use_by_emulator_are_deferred_not_errors() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);
    let card = a.make_folder_card("Mcd001_converted.ps2");
    write(&card.join("GAME1").join("save.dat"), b"g1");
    write(&card.join("GAME2").join("save.dat"), b"g2");
    write(&a.cards().join("Mcd002.ps2"), b"card");
    const GAME1: &str = "pcsx2/folders/Mcd001_converted/GAME1";

    let lock_game = lock_exclusively(&card.join("GAME1").join("save.dat"));
    let lock_card = lock_exclusively(&a.cards().join("Mcd002.ps2"));

    let s = a.sync().await;
    assert_eq!((s.uploaded, s.deferred, s.errors), (1, 2, 0), "{s:?}");
    assert!(s.error.is_none());
    assert_eq!(a.status(GAME1), UnitStatus::Deferred);
    assert_eq!(a.status(CARD.replace("Mcd001", "Mcd002").as_str()), UnitStatus::Deferred);
    // Nada do save travado foi para a nuvem.
    assert!(!bucket.keys().iter().any(|k| k.contains("GAME1")));
    // O loop de nova tentativa pega quando o emulador fecha.
    assert!(a.sync.should_retry_deferred(&a.source));

    drop(lock_game);
    drop(lock_card);
    let s = a.sync().await;
    assert_eq!((s.uploaded, s.deferred, s.errors), (2, 0, 0), "{s:?}");
    assert_eq!(a.status(GAME1), UnitStatus::Synced);
}

const GOW: &str = "pcsx2/folders/Mcd001_converted/BASCUS-97399GodOfWar";

/// Duas máquinas com o mesmo jogo sincronizado num cartão em pasta.
async fn two_machines_with_synced_game(bucket: &MemoryStorage) -> (Machine, Machine, PathBuf, PathBuf) {
    let a = Machine::new("PC-A", bucket);
    let b = Machine::new("PC-B", bucket);
    let game_a = a.make_folder_card("Mcd001_converted.ps2").join("BASCUS-97399GodOfWar");
    let game_b = b.make_folder_card("Mcd001_converted.ps2").join("BASCUS-97399GodOfWar");
    write(&game_a.join("data0.bin"), b"kratos");
    write(&game_a.join("icon.sys"), b"icon");
    a.sync().await;
    b.sync().await;
    assert!(game_b.join("data0.bin").is_file());
    (a, b, game_a, game_b)
}

fn tombstone_of(bucket: &MemoryStorage, unit_id: &str) -> RemoteManifest {
    let key = format!("test/{unit_id}/manifest.json");
    serde_json::from_slice(&bucket.objects.lock().unwrap()[&key]).unwrap()
}

#[tokio::test]
async fn delete_propagates_to_other_machines() {
    let bucket = MemoryStorage::default();
    let (a, b, game_a, game_b) = two_machines_with_synced_game(&bucket).await;

    let s = a.sync.delete_unit(&a.source, GOW).await.unwrap();
    assert_eq!(s.deleted, 1);
    assert!(!game_a.exists());
    assert!(a.root.join("backups").join("pcsx2").join("folders").is_dir(), "backup local");
    // Na nuvem fica só a lápide; os blobs foram removidos.
    assert!(tombstone_of(&bucket, GOW).deleted);
    assert!(!bucket.keys().iter().any(|k| k.contains("/blobs/")));

    // B não mexeu no jogo: apaga também, com backup.
    let s = b.sync().await;
    assert_eq!((s.deleted, s.conflicts, s.uploaded), (1, 0, 0));
    assert!(!game_b.exists());
    assert!(b.root.join("backups").join("pcsx2").join("folders").is_dir());

    // Nada volta nos próximos syncs.
    for m in [&a, &b] {
        let s = m.sync().await;
        assert_eq!((s.uploaded, s.downloaded, s.deleted, s.conflicts), (0, 0, 0, 0));
    }
    assert!(!game_a.exists() && !game_b.exists());
}

#[tokio::test]
async fn delete_vs_local_change_is_a_conflict() {
    let bucket = MemoryStorage::default();
    let (a, b, game_a, game_b) = two_machines_with_synced_game(&bucket).await;

    a.sync.delete_unit(&a.source, GOW).await.unwrap();
    write(&game_b.join("data0.bin"), b"kratos jogou mais");

    let s = b.sync().await;
    assert_eq!((s.conflicts, s.deleted), (1, 0));
    assert!(game_b.join("data0.bin").is_file(), "não apaga save alterado");
    assert!(b.state.load().unwrap().units[GOW].remote_deleted);

    // "Manter" reenvia, e a máquina A recebe o jogo de volta.
    b.sync.resolve_conflict(&b.source, GOW, Resolution::KeepLocal).await.unwrap();
    assert!(!tombstone_of(&bucket, GOW).deleted);
    assert_eq!(a.sync().await.downloaded, 1);
    assert_eq!(fs::read(game_a.join("data0.bin")).unwrap(), b"kratos jogou mais");
}

#[tokio::test]
async fn delete_vs_local_change_resolved_by_deleting() {
    let bucket = MemoryStorage::default();
    let (a, b, _game_a, game_b) = two_machines_with_synced_game(&bucket).await;

    a.sync.delete_unit(&a.source, GOW).await.unwrap();
    write(&game_b.join("data0.bin"), b"changed");
    b.sync().await;

    let s = b.sync.resolve_conflict(&b.source, GOW, Resolution::KeepCloud).await.unwrap();
    assert_eq!(s.deleted, 1);
    assert!(!game_b.exists());
}

#[tokio::test]
async fn delete_waits_for_emulator_to_close() {
    let bucket = MemoryStorage::default();
    let (a, b, _game_a, game_b) = two_machines_with_synced_game(&bucket).await;

    // Não exclui com o emulador aberto nesta máquina.
    a.process.0.store(true, Ordering::SeqCst);
    assert!(a.sync.delete_unit(&a.source, GOW).await.is_err());
    a.process.0.store(false, Ordering::SeqCst);
    a.sync.delete_unit(&a.source, GOW).await.unwrap();

    // Nas outras, a exclusão espera o emulador fechar.
    b.process.0.store(true, Ordering::SeqCst);
    assert_eq!(b.sync().await.deferred, 1);
    assert!(game_b.exists());
    b.process.0.store(false, Ordering::SeqCst);
    assert_eq!(b.sync().await.deleted, 1);
    assert!(!game_b.exists());
}

#[tokio::test]
async fn deleting_file_card_and_cloud_only_save() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);
    let b = Machine::new("PC-B", &bucket);
    write(&a.cards().join("Mcd002.ps2"), b"card");
    a.sync().await;
    const MCD2: &str = "pcsx2/memcards/Mcd002";

    // B nunca baixou: exclui só da nuvem, e A apaga a cópia dele no próximo sync.
    b.sync.delete_unit(&b.source, MCD2).await.unwrap();
    assert!(!b.cards().join("Mcd002.ps2").exists());
    assert_eq!(a.sync().await.deleted, 1);
    assert!(!a.cards().join("Mcd002.ps2").exists());
}

#[tokio::test]
async fn sync_without_cloud_config_fails_cleanly() {
    let bucket = MemoryStorage::default();
    let a = Machine::new("PC-A", &bucket);
    JsonFile::<AppSettings>::new(a.root.join("config.json")).write(&AppSettings::default()).unwrap();
    let err = a.sync.sync_source(&a.source, SyncTrigger::Manual).await.unwrap_err();
    assert!(matches!(err, crate::error::AppError::CloudNotConfigured));
}
