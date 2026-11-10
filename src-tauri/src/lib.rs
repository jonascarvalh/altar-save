mod application;
mod commands;
mod domain;
mod error;
mod infrastructure;
mod runtime;
mod state;

#[cfg(test)]
mod tests;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = state::AppState::build(app.handle())?;
            app.manage(state.clone());
            if let Err(err) = runtime::apply_automations(&state) {
                eprintln!("[save-nuvem] falha ao iniciar automações: {err}");
            }
            runtime::start_background_tasks(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_cloud_config,
            commands::clear_cloud_config,
            commands::test_connection,
            commands::update_pcsx2_settings,
            commands::detect_pcsx2,
            commands::pcsx2_candidates,
            commands::get_library,
            commands::sync_now,
            commands::resolve_conflict,
            commands::delete_save,
            commands::get_activity,
            commands::get_watch_status,
            commands::open_backups_folder,
            commands::open_saves_folder,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o save-nuvem");
}
