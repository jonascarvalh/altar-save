//! Monitora pastas de saves e dispara um callback após um período sem alterações (debounce).

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::fs_store::TEMP_MARKER;
use crate::error::{AppError, AppResult};

pub type ChangeCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
pub struct WatcherManager {
    watchers: Mutex<HashMap<String, RecommendedWatcher>>,
}

impl WatcherManager {
    /// (Re)inicia o monitoramento identificado por `key`.
    pub fn watch(&self, key: &str, paths: &[PathBuf], debounce: Duration, on_change: ChangeCallback) -> AppResult<()> {
        self.unwatch(key);

        let paths: Vec<&PathBuf> = paths.iter().filter(|p| p.is_dir()).collect();
        if paths.is_empty() {
            return Ok(());
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let mut watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
            if let Ok(event) = result
                && is_relevant(&event)
            {
                let _ = tx.send(());
            }
        })
        .map_err(|e| AppError::Internal(format!("Falha ao monitorar pasta: {e}")))?;

        for path in paths {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|e| AppError::Internal(format!("Falha ao monitorar {}: {e}", path.display())))?;
        }

        // A task termina sozinha quando o watcher é descartado (o canal fecha).
        tauri::async_runtime::spawn(async move {
            while rx.recv().await.is_some() {
                loop {
                    match tokio::time::timeout(debounce, rx.recv()).await {
                        Ok(Some(())) => continue,
                        Ok(None) => return,
                        Err(_) => break,
                    }
                }
                on_change();
            }
        });

        self.watchers.lock().unwrap_or_else(|e| e.into_inner()).insert(key.into(), watcher);
        Ok(())
    }

    pub fn unwatch(&self, key: &str) {
        self.watchers.lock().unwrap_or_else(|e| e.into_inner()).remove(key);
    }

    pub fn is_watching(&self, key: &str) -> bool {
        self.watchers.lock().unwrap_or_else(|e| e.into_inner()).contains_key(key)
    }
}

fn is_relevant(event: &Event) -> bool {
    matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_))
        && event.paths.iter().any(|p| !p.to_string_lossy().contains(TEMP_MARKER))
}
