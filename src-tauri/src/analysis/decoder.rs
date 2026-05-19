use crate::error::AppError;

pub fn decode_audio(path: &str) -> Result<Vec<f32>, AppError> {
    Err(AppError::NotImplemented("decode_audio".to_string()))
}
