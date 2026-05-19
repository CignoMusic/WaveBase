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
    _pool: State<'_, Arc<DbPool>>,
    _id: i64,
) -> Result<AudioFile, AppError> {
    Err(AppError::NotImplemented("get_file".to_string()))
}

#[tauri::command]
pub fn list_files(
    _pool: State<'_, Arc<DbPool>>,
    _limit: Option<usize>,
    _offset: Option<usize>,
) -> Result<Vec<AudioFile>, AppError> {
    Err(AppError::NotImplemented("list_files".to_string()))
}
