use std::sync::Arc;

use tauri::State;

use crate::db::models::Tag;
use crate::db::pool::DbPool;
use crate::error::AppError;

#[tauri::command]
pub fn add_tag(
    _pool: State<'_, Arc<DbPool>>,
    _file_id: i64,
    _tag_name: String,
) -> Result<(), AppError> {
    Err(AppError::NotImplemented("add_tag".to_string()))
}

#[tauri::command]
pub fn remove_tag(
    _pool: State<'_, Arc<DbPool>>,
    _file_id: i64,
    _tag_id: i64,
) -> Result<(), AppError> {
    Err(AppError::NotImplemented("remove_tag".to_string()))
}

#[tauri::command]
pub fn list_tags(
    _pool: State<'_, Arc<DbPool>>,
    _file_id: Option<i64>,
) -> Result<Vec<Tag>, AppError> {
    Err(AppError::NotImplemented("list_tags".to_string()))
}
