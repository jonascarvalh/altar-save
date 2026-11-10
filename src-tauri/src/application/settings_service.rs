use std::sync::Arc;

use crate::{
    domain::{
        models::{AppSettings, CloudConfig, Pcsx2Settings},
        ports::SettingsRepository,
    },
    error::AppResult,
};

pub struct SettingsService {
    repo: Arc<dyn SettingsRepository>,
}

impl SettingsService {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
    }

    pub fn get(&self) -> AppResult<AppSettings> {
        self.repo.load()
    }

    pub fn save_cloud(&self, config: CloudConfig) -> AppResult<AppSettings> {
        let config = normalize(config);
        config.validate()?;
        self.update(|s| s.cloud = Some(config))
    }

    pub fn clear_cloud(&self) -> AppResult<AppSettings> {
        self.update(|s| s.cloud = None)
    }

    pub fn update_pcsx2(&self, pcsx2: Pcsx2Settings) -> AppResult<AppSettings> {
        let data_dir = pcsx2.data_dir.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
        self.update(|s| s.pcsx2 = Pcsx2Settings { data_dir, ..pcsx2 })
    }

    fn update(&self, change: impl FnOnce(&mut AppSettings)) -> AppResult<AppSettings> {
        let mut settings = self.repo.load()?;
        change(&mut settings);
        self.repo.save(&settings)?;
        Ok(settings)
    }
}

fn normalize(c: CloudConfig) -> CloudConfig {
    CloudConfig {
        provider: c.provider,
        account_id: c.account_id.trim().into(),
        region: c.region.trim().into(),
        endpoint: c.endpoint.trim().trim_end_matches('/').into(),
        bucket: c.bucket.trim().into(),
        prefix: c.prefix.trim().trim_matches('/').into(),
        access_key_id: c.access_key_id.trim().into(),
        secret_access_key: c.secret_access_key.trim().into(),
    }
}
