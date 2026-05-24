use std::sync::Arc;

use tauri::State;

use crate::db::models::AudioFile;
use crate::db::pool::DbPool;
use crate::error::AppError;

#[tauri::command]
pub fn search_files(
    _pool: State<'_, Arc<DbPool>>,
    _query: String,
) -> Result<Vec<AudioFile>, AppError> {
    Err(AppError::NotImplemented("search_files".to_string()))
}

#[tauri::command]
pub fn get_file(
    pool: State<'_, Arc<DbPool>>,
    path: String,
) -> Result<AudioFile, AppError> {
    let conn = pool.get()?;
    let file = conn.query_row(
        "SELECT id, path, filename, folder_path, extension, size_bytes, modified_at,
                duration_secs, track_name, bpm, key, artist, bpm_analyzed, key_analyzed,
                created_at, updated_at
         FROM audio_files WHERE path = ?1",
        rusqlite::params![path],
        |row| {
            Ok(AudioFile {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                folder_path: row.get(3)?,
                extension: row.get(4)?,
                size_bytes: row.get(5)?,
                modified_at: row.get(6)?,
                duration_secs: row.get(7)?,
                track_name: row.get(8)?,
                bpm: row.get(9)?,
                key: row.get(10)?,
                artist: row.get(11)?,
                bpm_analyzed: row.get::<_, i32>(12)? != 0,
                key_analyzed: row.get::<_, i32>(13)? != 0,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        },
    )?;
    Ok(file)
}

#[tauri::command]
pub fn list_files(
    pool: State<'_, Arc<DbPool>>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<AudioFile>, AppError> {
    let conn = pool.get()?;
    let limit = limit.unwrap_or(1000);
    let offset = offset.unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT id, path, filename, folder_path, extension, size_bytes, modified_at,
                duration_secs, track_name, bpm, key, artist, bpm_analyzed, key_analyzed,
                created_at, updated_at
         FROM audio_files ORDER BY filename LIMIT ?1 OFFSET ?2",
    )?;

    let files = stmt
        .query_map(rusqlite::params![limit as i64, offset as i64], |row| {
            Ok(AudioFile {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                folder_path: row.get(3)?,
                extension: row.get(4)?,
                size_bytes: row.get(5)?,
                modified_at: row.get(6)?,
                duration_secs: row.get(7)?,
                track_name: row.get(8)?,
                bpm: row.get(9)?,
                key: row.get(10)?,
                artist: row.get(11)?,
                bpm_analyzed: row.get::<_, i32>(12)? != 0,
                key_analyzed: row.get::<_, i32>(13)? != 0,
                created_at: row.get(14)?,
                updated_at: row.get(15)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(files)
}
