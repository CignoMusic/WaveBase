use tauri::State;
use crate::error::AppError;
use crate::playback::player::{AudioPlayer, PlaybackStatus};

#[tauri::command]
pub fn play_audio(
    player: State<'_, AudioPlayer>,
    path: String,
) -> Result<PlaybackStatus, AppError> {
    player.play(&path)
}

#[tauri::command]
pub fn toggle_playback(
    player: State<'_, AudioPlayer>,
    path: String,
) -> Result<PlaybackStatus, AppError> {
    player.toggle(&path)
}

#[tauri::command]
pub fn pause_audio(
    player: State<'_, AudioPlayer>,
) -> Result<PlaybackStatus, AppError> {
    player.pause()
}

#[tauri::command]
pub fn resume_audio(
    player: State<'_, AudioPlayer>,
) -> Result<PlaybackStatus, AppError> {
    player.resume()
}

#[tauri::command]
pub fn stop_audio(
    player: State<'_, AudioPlayer>,
) -> Result<PlaybackStatus, AppError> {
    player.stop()
}

#[tauri::command]
pub fn get_playback_status(
    player: State<'_, AudioPlayer>,
) -> Result<PlaybackStatus, AppError> {
    Ok(player.status())
}

#[tauri::command]
pub fn set_volume(
    player: State<'_, AudioPlayer>,
    volume: f32,
) -> Result<(), AppError> {
    player.set_volume(volume)
}
