//! Contratos que a camada de aplicação usa e a infraestrutura implementa.

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use async_trait::async_trait;

use super::models::*;
use crate::error::AppResult;

/// Armazenamento de objetos compatível com S3.
#[async_trait]
pub trait CloudStorage: Send + Sync {
    async fn check_bucket(&self) -> AppResult<()>;
    async fn put_object(&self, key: &str, body: Vec<u8>, metadata: &[(&str, String)]) -> AppResult<()>;
    /// `Ok(None)` quando o objeto não existe.
    async fn get_object(&self, key: &str) -> AppResult<Option<Vec<u8>>>;
    async fn delete_object(&self, key: &str) -> AppResult<()>;
    async fn list_keys(&self, prefix: &str) -> AppResult<Vec<String>>;
}

pub trait CloudStorageFactory: Send + Sync {
    fn create(&self, config: &CloudConfig) -> AppResult<Arc<dyn CloudStorage>>;
}

pub trait SettingsRepository: Send + Sync {
    fn load(&self) -> AppResult<AppSettings>;
    fn save(&self, settings: &AppSettings) -> AppResult<()>;
}

pub trait SyncStateRepository: Send + Sync {
    fn load(&self) -> AppResult<SyncStateFile>;
    fn save(&self, state: &SyncStateFile) -> AppResult<()>;
}

pub trait ActivityLog: Send + Sync {
    fn append(&self, entry: ActivityEntry) -> AppResult<()>;
    fn recent(&self, limit: usize) -> AppResult<Vec<ActivityEntry>>;
}

/// Leitura e escrita dos arquivos locais de uma unidade.
pub trait LocalUnitStore: Send + Sync {
    fn read_file(&self, unit: &SyncUnit, relative_path: &str) -> AppResult<Vec<u8>>;
    /// Substitui o conteúdo local da unidade de forma atômica.
    fn replace_unit(&self, unit: &SyncUnit, files: Vec<(FileEntry, Vec<u8>)>) -> AppResult<()>;
    /// Apaga a cópia local da unidade (o chamador faz backup antes).
    fn remove_unit(&self, unit: &SyncUnit) -> AppResult<()>;
}

pub trait BackupStore: Send + Sync {
    /// Copia o estado local atual da unidade e mantém apenas os `keep` mais recentes.
    fn backup(&self, unit: &SyncUnit, keep: usize) -> AppResult<Option<PathBuf>>;
}

pub trait ProcessMonitor: Send + Sync {
    /// `names` são trechos (minúsculos) do nome do executável.
    fn is_any_running(&self, names: &[&str]) -> bool;
}

pub trait SyncEventSink: Send + Sync {
    fn emit(&self, event: SyncEvent);
}

/// Região da nuvem onde podem existir unidades que ainda não existem localmente.
#[derive(Debug, Clone)]
pub struct RemoteScope {
    /// Prefixo do ID das unidades, ex.: `pcsx2/folders/Mcd001/`.
    pub id_prefix: String,
    pub kind: UnitKind,
    pub group: Option<String>,
    pub local_parent: PathBuf,
    /// Sufixo do caminho local, ex.: `.ps2` para memory cards em arquivo.
    pub local_suffix: String,
}

#[derive(Debug, Clone, Default)]
pub struct SourceScan {
    pub units: Vec<SyncUnit>,
    pub remote_scopes: Vec<RemoteScope>,
}

/// Um emulador (ou outra origem de saves).
pub trait SaveSource: Send + Sync {
    fn id(&self) -> &str;
    fn info(&self) -> SourceInfo;
    fn scan(&self) -> AppResult<SourceScan>;
    fn watch_paths(&self) -> Vec<PathBuf>;
    fn watch_debounce(&self) -> Duration;
    fn process_names(&self) -> &'static [&'static str];

    /// Capas locais (somente leitura) por ID de unidade. Usado apenas para exibição.
    fn find_covers(&self, _units: &[SyncUnit]) -> HashMap<String, PathBuf> {
        HashMap::new()
    }
}

pub struct RegisteredSource {
    pub source: Arc<dyn SaveSource>,
    pub enabled: bool,
    pub auto_sync: bool,
}

pub trait SourceProvider: Send + Sync {
    fn sources(&self, settings: &AppSettings) -> Vec<RegisteredSource>;
}
