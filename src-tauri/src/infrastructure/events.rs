use tauri::{AppHandle, Emitter};

use crate::domain::{models::SyncEvent, ports::SyncEventSink};

pub const SYNC_EVENT: &str = "sync://event";

/// Publica eventos de sync para o frontend.
pub struct TauriEventSink {
    app: AppHandle,
}

impl TauriEventSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl SyncEventSink for TauriEventSink {
    fn emit(&self, event: SyncEvent) {
        if let Err(err) = self.app.emit(SYNC_EVENT, event) {
            eprintln!("[save-nuvem] falha ao emitir evento: {err}");
        }
    }
}

/// Descarta eventos (testes).
#[cfg(test)]
pub struct NoopEventSink;

#[cfg(test)]
impl SyncEventSink for NoopEventSink {
    fn emit(&self, _event: SyncEvent) {}
}
