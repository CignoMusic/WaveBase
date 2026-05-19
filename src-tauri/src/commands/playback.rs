use crate::error::AppError;

#[tauri::command]
pub fn play_audio(_path: String) -> Result<(), AppError> {
    Err(AppError::NotImplemented("play_audio".to_string()))
}

#[tauri::command]
pub fn stop_audio() -> Result<(), AppError> {
    Err(AppError::NotImplemented("stop_audio".to_string()))
}

#[tauri::command]
pub fn pause_audio() -> Result<(), AppError> {
    Err(AppError::NotImplemented("pause_audio".to_string()))
}
