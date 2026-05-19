use std::sync::Arc;

use tauri::State;

use crate::db::models::ScanProgress;
use crate::db::pool::DbPool;
use crate::error::AppError;
use crate::scanner::filesystem::{self, ScannedFile};

#[tauri::command]
pub fn scan_directory(
    pool: State<'_, Arc<DbPool>>,
    path: String,
) -> Result<Vec<ScannedFile>, AppError> {
    let dir = std::path::Path::new(&path);
    let files = filesystem::scan_directory(dir)?;

    let conn = pool.get()?;
    for file in &files {
        let folder_path = std::path::Path::new(&file.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        conn.execute(
            "INSERT OR IGNORE INTO audio_files (path, filename, folder_path, extension, size_bytes, modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![file.path, file.filename, folder_path, file.extension, file.size_bytes as i64, file.modified_at],
        )?;
    }

    Ok(files)
}

#[tauri::command]
pub fn scan_status() -> Result<ScanProgress, AppError> {
    Err(AppError::NotImplemented("scan_status".to_string()))
}
