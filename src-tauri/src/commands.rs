//! Comandos expostos ao frontend. São finos: só delegam para os casos de uso.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    application::{
        library::{self, SourceOverview},
        test_connection::{self, ConnectionReport},
    },
    domain::{
        models::{ActivityEntry, AppSettings, CloudConfig, Pcsx2Settings, Resolution, SourceInfo, SyncSummary, SyncTrigger},
        ports::SaveSource as _,
    },
    error::{AppError, AppResult},
    infrastructure::sources::pcsx2::{self, Pcsx2Source},
    runtime,
    state::AppState,
};

// --- Configurações ---------------------------------------------------------

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    state.settings.get()
}

#[tauri::command]
pub async fn save_cloud_config(state: State<'_, AppState>, config: CloudConfig) -> AppResult<AppSettings> {
    let settings = state.settings.save_cloud(config)?;
    runtime::apply_automations(&state)?;
    Ok(settings)
}

#[tauri::command]
pub async fn clear_cloud_config(state: State<'_, AppState>) -> AppResult<AppSettings> {
    let settings = state.settings.clear_cloud()?;
    runtime::apply_automations(&state)?;
    Ok(settings)
}

#[tauri::command]
pub async fn test_connection(state: State<'_, AppState>, config: CloudConfig) -> AppResult<ConnectionReport> {
    test_connection::run(state.storage_factory.as_ref(), &config).await
}

#[tauri::command]
pub async fn update_pcsx2_settings(state: State<'_, AppState>, settings: Pcsx2Settings) -> AppResult<AppSettings> {
    let updated = state.settings.update_pcsx2(settings)?;
    runtime::apply_automations(&state)?;
    Ok(updated)
}

/// Pré-visualiza a detecção do PCSX2 para uma pasta antes de salvar.
#[tauri::command]
pub async fn detect_pcsx2(data_dir: Option<String>) -> AppResult<SourceInfo> {
    let settings = Pcsx2Settings { data_dir, ..Default::default() };
    Ok(Pcsx2Source::new(&settings).info())
}

/// Instalações do PCSX2 encontradas: portáteis em execução e a pasta padrão em Documentos.
#[tauri::command]
pub async fn pcsx2_candidates(state: State<'_, AppState>) -> AppResult<Vec<SourceInfo>> {
    let executables = state.processes.running_executables(pcsx2::process_names());
    Ok(pcsx2::candidate_data_dirs(&executables)
        .into_iter()
        .map(|dir| {
            let settings = Pcsx2Settings { data_dir: Some(dir.display().to_string()), ..Default::default() };
            Pcsx2Source::new(&settings).info()
        })
        .collect())
}

// --- Biblioteca e sync -----------------------------------------------------

#[tauri::command]
pub async fn get_library(app: AppHandle, state: State<'_, AppState>) -> AppResult<Vec<SourceOverview>> {
    let repo = state.sync.state().clone();
    let overviews = state
        .registered_sources()?
        .iter()
        .map(|registered| library::overview(registered, repo.as_ref()))
        .collect::<AppResult<Vec<_>>>()?;

    // A interface só consegue exibir arquivos locais liberados explicitamente:
    // libera apenas as capas encontradas, nunca pastas inteiras.
    let scope = app.asset_protocol_scope();
    for cover in overviews.iter().flat_map(|o| &o.units).filter_map(|u| u.cover_path.as_ref()) {
        if let Err(err) = scope.allow_file(cover) {
            eprintln!("[save-nuvem] capa indisponível {}: {err}", cover.display());
        }
    }
    Ok(overviews)
}

#[tauri::command]
pub async fn sync_now(state: State<'_, AppState>, source_id: String) -> AppResult<SyncSummary> {
    runtime::run_sync(&state, &source_id, SyncTrigger::Manual).await
}

#[tauri::command]
pub async fn resolve_conflict(
    state: State<'_, AppState>,
    unit_id: String,
    resolution: Resolution,
) -> AppResult<SyncSummary> {
    let source_id = unit_id
        .split('/')
        .next()
        .ok_or_else(|| AppError::Validation("ID de save inválido.".into()))?;
    let registered = state.source(source_id)?;
    state.sync.resolve_conflict(registered.source.as_ref(), &unit_id, resolution).await
}

#[tauri::command]
pub async fn delete_save(state: State<'_, AppState>, unit_id: String) -> AppResult<SyncSummary> {
    let source_id = unit_id
        .split('/')
        .next()
        .ok_or_else(|| AppError::Validation("ID de save inválido.".into()))?;
    let registered = state.source(source_id)?;
    state.sync.delete_unit(registered.source.as_ref(), &unit_id).await
}

#[tauri::command]
pub async fn get_activity(state: State<'_, AppState>, limit: Option<usize>) -> AppResult<Vec<ActivityEntry>> {
    state.activity.recent(limit.unwrap_or(100))
}

#[tauri::command]
pub async fn get_watch_status(state: State<'_, AppState>) -> AppResult<Vec<(String, bool)>> {
    Ok(state
        .registered_sources()?
        .iter()
        .map(|r| {
            let id = r.source.id().to_string();
            let watching = state.watchers.is_watching(&id);
            (id, watching)
        })
        .collect())
}

// --- Sistema ---------------------------------------------------------------

#[tauri::command]
pub async fn open_backups_folder(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    std::fs::create_dir_all(&state.backups_dir)?;
    open_path(&app, &state.backups_dir.display().to_string())
}

#[tauri::command]
pub async fn open_saves_folder(app: AppHandle, state: State<'_, AppState>, source_id: String) -> AppResult<()> {
    let registered = state.source(&source_id)?;
    let dir = registered
        .source
        .info()
        .saves_dir
        .ok_or_else(|| AppError::NotFound("Pasta de saves não encontrada.".into()))?;
    open_path(&app, &dir)
}

fn open_path(app: &AppHandle, path: &str) -> AppResult<()> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| AppError::Internal(format!("Não foi possível abrir a pasta: {e}")))
}
