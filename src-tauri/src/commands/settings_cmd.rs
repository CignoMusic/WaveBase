use std::sync::Arc;

use tauri::State;

use crate::config;
use crate::db::models::ScanRoot;
use crate::db::pool::DbPool;
use crate::error::AppError;

#[tauri::command]
pub fn list_scan_roots(
    pool: State<'_, Arc<DbPool>>,
) -> Result<Vec<ScanRoot>, AppError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, path FROM scan_roots ORDER BY path")?;
    let roots = stmt
        .query_map([], |row| {
            Ok(ScanRoot {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(roots)
}

#[tauri::command]
pub fn add_scan_root(
    pool: State<'_, Arc<DbPool>>,
    path: String,
) -> Result<ScanRoot, AppError> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT OR IGNORE INTO scan_roots (path) VALUES (?1)",
        rusqlite::params![&path],
    )?;

    let root = conn.query_row(
        "SELECT id, path FROM scan_roots WHERE path = ?1",
        rusqlite::params![&path],
        |row| {
            Ok(ScanRoot {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        },
    )?;

    Ok(root)
}

#[tauri::command]
pub fn remove_scan_root(
    pool: State<'_, Arc<DbPool>>,
    id: i64,
    path: String,
) -> Result<(), AppError> {
    let conn = pool.get()?;
    conn.execute(
        "DELETE FROM scan_roots WHERE id = ?1",
        rusqlite::params![id],
    )?;
    // Delete all audio files whose path starts with this directory
    // Escape _ and % in the path so they aren't treated as LIKE wildcards
    let escaped = path.replace('\\', "\\\\")
                     .replace('%', "\\%")
                     .replace('_', "\\_");
    let like_pattern = format!("{}%", escaped);
    conn.execute(
        "DELETE FROM audio_files WHERE path LIKE ?1 ESCAPE '\\'",
        rusqlite::params![like_pattern],
    )?;
    Ok(())
}

#[tauri::command]
pub fn get_database_size(
) -> Result<i64, AppError> {
    let data_dir = config::ensure_data_dir();
    let db_path = config::db_path(&data_dir);
    let meta = std::fs::metadata(&db_path)?;
    Ok(meta.len() as i64)
}

#[tauri::command]
pub fn clear_database(
    pool: State<'_, Arc<DbPool>>,
) -> Result<i64, AppError> {
    let conn = pool.get()?;
    conn.execute_batch(
        "DELETE FROM file_tags;
         DELETE FROM audio_files;
         VACUUM;",
    )?;
    // Return the new DB size after VACUUM
    drop(conn);
    let data_dir = config::ensure_data_dir();
    let db_path = config::db_path(&data_dir);
    let meta = std::fs::metadata(&db_path)?;
    Ok(meta.len() as i64)
}
