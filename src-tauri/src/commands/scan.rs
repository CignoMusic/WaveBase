use std::sync::Arc;

use tauri::State;

use crate::db::models::ScanProgress;
use crate::db::pool::DbPool;
use crate::error::AppError;

#[tauri::command]
pub fn scan_directory(
    _pool: State<'_, Arc<DbPool>>,
    _path: String,
) -> Result<(), AppError> {
    Err(AppError::NotImplemented("scan_directory".to_string()))
}

#[tauri::command]
pub fn scan_status() -> Result<ScanProgress, AppError> {
    Err(AppError::NotImplemented("scan_status".to_string()))
}
