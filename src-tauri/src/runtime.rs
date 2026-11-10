//! Automações em segundo plano: watchers, sync ao iniciar e nova tentativa de downloads adiados.

use std::{sync::Arc, time::Duration};

use crate::{
    domain::models::{SyncEvent, SyncSummary, SyncTrigger},
    error::{AppError, AppResult},
    state::AppState,
};

const RETRY_INTERVAL: Duration = Duration::from_secs(30);

/// Liga ou desliga os watchers conforme as configurações atuais.
pub fn apply_automations(state: &AppState) -> AppResult<()> {
    let settings = state.settings.get()?;
    for registered in state.sources.sources(&settings) {
        let id = registered.source.id().to_string();
        if registered.enabled && registered.auto_sync && settings.cloud.is_some() {
            let app_state = state.clone();
            let source_id = id.clone();
            state.watchers.watch(
                &id,
                &registered.source.watch_paths(),
                registered.source.watch_debounce(),
                Arc::new(move || {
                    // Atualiza a interface mesmo que o sync em seguida falhe (ex.: nuvem fora do ar).
                    app_state.events.emit(SyncEvent::LocalChanged { source_id: source_id.clone() });
                    spawn_sync(app_state.clone(), source_id.clone(), SyncTrigger::Watcher);
                }),
            )?;
        } else {
            state.watchers.unwatch(&id);
        }
    }
    Ok(())
}

pub async fn run_sync(state: &AppState, source_id: &str, trigger: SyncTrigger) -> AppResult<SyncSummary> {
    let registered = state.source(source_id)?;
    if !registered.enabled {
        return Err(AppError::Validation("Esta automação está desativada.".into()));
    }
    state.sync.sync_source(registered.source.as_ref(), trigger).await
}

pub fn spawn_sync(state: AppState, source_id: String, trigger: SyncTrigger) {
    tauri::async_runtime::spawn(async move {
        if let Err(err) = run_sync(&state, &source_id, trigger).await {
            eprintln!("[altar-save] sync de {source_id} falhou: {err}");
        }
    });
}

/// Sync inicial das automações ativas e loop de nova tentativa para downloads adiados.
pub fn start_background_tasks(state: AppState) {
    tauri::async_runtime::spawn(async move {
        for_each_auto_source(&state, |_| true, SyncTrigger::Startup).await;

        let mut interval = tokio::time::interval(RETRY_INTERVAL);
        interval.tick().await;
        loop {
            interval.tick().await;
            let sync = state.sync.clone();
            for_each_auto_source(&state, move |r| sync.should_retry_deferred(r), SyncTrigger::Retry).await;
        }
    });
}

async fn for_each_auto_source(
    state: &AppState,
    filter: impl Fn(&dyn crate::domain::ports::SaveSource) -> bool,
    trigger: SyncTrigger,
) {
    let Ok(settings) = state.settings.get() else { return };
    if settings.cloud.is_none() {
        return;
    }
    for registered in state.sources.sources(&settings) {
        if registered.enabled && registered.auto_sync && filter(registered.source.as_ref()) {
            if let Err(err) = state.sync.sync_source(registered.source.as_ref(), trigger).await {
                eprintln!("[altar-save] sync automático falhou: {err}");
            }
        }
    }
}
