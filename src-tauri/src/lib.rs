mod analysis;
mod commands;
mod config;
mod db;
mod error;
mod playback;
mod scanner;

use std::sync::{Arc, Mutex};

use db::models::TagProgress;
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

    let tag_progress = Arc::new(commands::scan::BackgroundTagState {
        progress: Arc::new(Mutex::new(TagProgress {
            total: 0,
            processed: 0,
            status: "idle".to_string(),
        })),
    });

    tauri::Builder::default()
        .manage(pool)
        .manage(tag_progress)
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let player = AudioPlayer::new();
            app.manage(player);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Scan
            commands::scan::scan_directory,
            commands::scan::get_tag_progress,
            commands::scan::scan_status,
            // Playback
            commands::playback::play_audio,
            commands::playback::toggle_playback,
            commands::playback::pause_audio,
            commands::playback::resume_audio,
            commands::playback::stop_audio,
            commands::playback::get_playback_status,
            commands::playback::set_volume,
            commands::playback::seek_audio,
            commands::playback::set_duration,
            commands::playback::store_track_duration,
            commands::playback::get_waveform_data,
            // Library
            commands::library::search_files,
            commands::library::get_file,
            commands::library::list_files,
            // Tags
            commands::tags::add_tag,
            commands::tags::remove_tag,
            commands::tags::list_file_tags,
            commands::tags::get_all_tags,
            commands::tags::get_pinned_tags,
            commands::tags::toggle_tag_pin,
            commands::tags::create_tag,
            commands::tags::delete_tag,
            commands::tags::filter_files_by_tag_names,
            commands::tags::get_tag_file_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
