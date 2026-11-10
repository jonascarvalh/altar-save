//! Composição: cria as implementações concretas e injeta nos casos de uso.

use std::{fs, path::PathBuf, sync::Arc};

use tauri::{AppHandle, Manager};

use crate::{
    application::{
        settings_service::SettingsService,
        sync_service::{SyncDeps, SyncService},
    },
    domain::{
        models::{AppSettings, SyncStateFile},
        ports::{ActivityLog, CloudStorageFactory, RegisteredSource, SourceProvider, SyncEventSink},
    },
    error::{AppError, AppResult},
    infrastructure::{
        backups::FsBackupStore,
        events::TauriEventSink,
        fs_store::FsUnitStore,
        persistence::{JsonActivityLog, JsonFile},
        process::SysinfoProcessMonitor,
        sources::DefaultSourceProvider,
        storage::s3_storage::S3StorageFactory,
        watcher::WatcherManager,
    },
};

const MAX_ACTIVITY_ENTRIES: usize = 300;

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<SettingsService>,
    pub sync: Arc<SyncService>,
    pub sources: Arc<dyn SourceProvider>,
    pub activity: Arc<dyn ActivityLog>,
    pub storage_factory: Arc<dyn CloudStorageFactory>,
    pub watchers: Arc<WatcherManager>,
    pub processes: Arc<SysinfoProcessMonitor>,
    pub events: Arc<dyn SyncEventSink>,
    pub backups_dir: PathBuf,
}

impl AppState {
    pub fn build(app: &AppHandle) -> AppResult<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Internal(format!("Pasta de dados indisponível: {e}")))?;
        fs::create_dir_all(&data_dir)?;
        let backups_dir = data_dir.join("backups");

        let settings_repo = Arc::new(JsonFile::<AppSettings>::new(data_dir.join("config.json")));
        let state_repo = Arc::new(JsonFile::<SyncStateFile>::new(data_dir.join("sync-state.json")));
        let activity: Arc<dyn ActivityLog> =
            Arc::new(JsonActivityLog::new(data_dir.join("activity.json"), MAX_ACTIVITY_ENTRIES));
        let storage_factory: Arc<dyn CloudStorageFactory> = Arc::new(S3StorageFactory);
        let processes = Arc::new(SysinfoProcessMonitor::default());
        let events: Arc<dyn SyncEventSink> = Arc::new(TauriEventSink::new(app.clone()));

        let sync = SyncService::new(SyncDeps {
            storage_factory: storage_factory.clone(),
            settings: settings_repo.clone(),
            state: state_repo,
            activity: activity.clone(),
            local: Arc::new(FsUnitStore),
            backups: Arc::new(FsBackupStore::new(&backups_dir)),
            process: processes.clone(),
            events: events.clone(),
            machine: machine_name(),
        });

        Ok(Self {
            settings: Arc::new(SettingsService::new(settings_repo)),
            sync: Arc::new(sync),
            sources: Arc::new(DefaultSourceProvider),
            activity,
            storage_factory,
            watchers: Arc::new(WatcherManager::default()),
            processes,
            events,
            backups_dir,
        })
    }

    pub fn registered_sources(&self) -> AppResult<Vec<RegisteredSource>> {
        Ok(self.sources.sources(&self.settings.get()?))
    }

    pub fn source(&self, source_id: &str) -> AppResult<RegisteredSource> {
        self.registered_sources()?
            .into_iter()
            .find(|s| s.source.id() == source_id)
            .ok_or_else(|| AppError::NotFound(format!("Fonte '{source_id}' desconhecida.")))
    }
}

fn machine_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "desconhecido".into())
}
