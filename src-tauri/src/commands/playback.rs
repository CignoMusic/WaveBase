use std::sync::Arc;

use tauri::State;

use crate::db::pool::DbPool;
use crate::error::AppError;
use crate::playback::player::{AudioPlayer, PlaybackStatus};
use crate::playback::waveform::WaveformData;

fn get_stored_duration(pool: &DbPool, path: &str) -> Result<Option<f64>, AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare_cached(
        "SELECT duration_secs FROM audio_files WHERE path = ?1 AND duration_secs IS NOT NULL",
    )?;
    let mut rows = stmt.query(rusqlite::params![path])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

fn set_stored_duration(pool: &DbPool, path: &str, duration: f64) -> Result<(), AppError> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE audio_files SET duration_secs = ?1, updated_at = datetime('now') WHERE path = ?2",
        rusqlite::params![duration, path],
    )?;
    Ok(())
}

#[tauri::command]
pub fn play_audio(
    pool: State<'_, Arc<DbPool>>,
    player: State<'_, AudioPlayer>,
    path: String,
) -> Result<PlaybackStatus, AppError> {
    let status = player.play(&path)?;
    if let Some(d) = get_stored_duration(&pool, &path)? {
        player.set_duration(d)?;
        return Ok(player.status());
    }
    Ok(status)
}

#[tauri::command]
pub fn toggle_playback(
    pool: State<'_, Arc<DbPool>>,
    player: State<'_, AudioPlayer>,
    path: String,
) -> Result<PlaybackStatus, AppError> {
    let status = player.toggle(&path)?;
    if let Some(d) = get_stored_duration(&pool, &path)? {
        player.set_duration(d)?;
        return Ok(player.status());
    }
    Ok(status)
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

#[tauri::command]
pub fn seek_audio(
    player: State<'_, AudioPlayer>,
    position: f64,
) -> Result<PlaybackStatus, AppError> {
    player.seek(position)
}

#[tauri::command]
pub fn set_duration(
    player: State<'_, AudioPlayer>,
    duration: f64,
) -> Result<(), AppError> {
    player.set_duration(duration)
}

#[tauri::command]
pub fn store_track_duration(
    pool: State<'_, Arc<DbPool>>,
    path: String,
    duration: f64,
) -> Result<(), AppError> {
    set_stored_duration(&pool, &path, duration)
}

#[tauri::command]
pub async fn get_waveform_data(path: String, bars: usize) -> Result<WaveformData, AppError> {
    tokio::task::spawn_blocking(move || {
        crate::playback::waveform::compute_waveform_peaks(&path, bars)
    })
    .await
    .map_err(|e| AppError::Playback(format!("Waveform thread panicked: {}", e)))?
}
