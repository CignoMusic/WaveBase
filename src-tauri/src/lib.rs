mod commands;
mod config;
mod db;
mod error;
mod playback;
mod scanner;

use std::sync::Arc;

use db::pool::DbPool;
use playback::player::AudioPlayer;
use tauri::Manager;

pub use error::AppError;

pub fn run() {
    let data_dir = config::ensure_data_dir();
    let db_path = config::db_path(&data_dir);
    let pool: Arc<DbPool> = Arc::new(db::pool::create_pool(&db_path));

    db::migrations::run_migrations(&pool.get().expect("Failed to get connection for migrations"))
        .expect("Failed to run database migrations");

    tauri::Builder::default()
        .manage(pool)
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let player = AudioPlayer::new();
            app.manage(player);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan::scan_directory,
            commands::scan::scan_status,
            commands::playback::play_audio,
            commands::playback::toggle_playback,
            commands::playback::pause_audio,
            commands::playback::resume_audio,
            commands::playback::stop_audio,
            commands::playback::get_playback_status,
            commands::playback::set_volume,
            commands::library::search_files,
            commands::library::get_file,
            commands::library::list_files,
            commands::tags::add_tag,
            commands::tags::remove_tag,
            commands::tags::list_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
