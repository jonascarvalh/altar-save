//! Persistência local em arquivos JSON (escrita atômica via arquivo temporário + rename).

use std::{
    fs, io,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Mutex,
};

use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    domain::{
        models::{ActivityEntry, AppSettings, SyncStateFile},
        ports::{ActivityLog, SettingsRepository, SyncStateRepository},
    },
    error::AppResult,
};

pub struct JsonFile<T> {
    path: PathBuf,
    lock: Mutex<()>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Serialize + DeserializeOwned + Default> JsonFile<T> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), lock: Mutex::new(()), _marker: PhantomData }
    }

    pub fn read(&self) -> AppResult<T> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.read_unlocked()
    }

    pub fn write(&self, value: &T) -> AppResult<()> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.write_unlocked(value)
    }

    pub fn update<R>(&self, change: impl FnOnce(&mut T) -> R) -> AppResult<R> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut value = self.read_unlocked()?;
        let result = change(&mut value);
        self.write_unlocked(&value)?;
        Ok(result)
    }

    fn read_unlocked(&self) -> AppResult<T> {
        match fs::read(&self.path) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(value) => Ok(value),
                Err(err) => {
                    // Preserva o arquivo corrompido para análise e segue com o padrão.
                    let stamp = Utc::now().format("%Y%m%d%H%M%S");
                    let _ = fs::rename(&self.path, self.path.with_extension(format!("corrupt-{stamp}.json")));
                    eprintln!("[altar-save] {} inválido ({err}); usando valores padrão.", self.path.display());
                    Ok(T::default())
                }
            },
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(T::default()),
            Err(err) => Err(err.into()),
        }
    }

    fn write_unlocked(&self, value: &T) -> AppResult<()> {
        write_atomic(&self.path, &serde_json::to_vec_pretty(value)?)
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

impl SettingsRepository for JsonFile<AppSettings> {
    fn load(&self) -> AppResult<AppSettings> {
        self.read()
    }

    fn save(&self, settings: &AppSettings) -> AppResult<()> {
        self.write(settings)
    }
}

impl SyncStateRepository for JsonFile<SyncStateFile> {
    fn load(&self) -> AppResult<SyncStateFile> {
        self.read()
    }

    fn save(&self, state: &SyncStateFile) -> AppResult<()> {
        self.write(state)
    }
}

pub struct JsonActivityLog {
    file: JsonFile<Vec<ActivityEntry>>,
    max_entries: usize,
}

impl JsonActivityLog {
    pub fn new(path: impl Into<PathBuf>, max_entries: usize) -> Self {
        Self { file: JsonFile::new(path), max_entries }
    }
}

impl ActivityLog for JsonActivityLog {
    fn append(&self, entry: ActivityEntry) -> AppResult<()> {
        let max = self.max_entries;
        self.file.update(|entries| {
            entries.push(entry);
            if entries.len() > max {
                let excess = entries.len() - max;
                entries.drain(..excess);
            }
        })
    }

    fn recent(&self, limit: usize) -> AppResult<Vec<ActivityEntry>> {
        Ok(self.file.read()?.into_iter().rev().take(limit).collect())
    }
}
