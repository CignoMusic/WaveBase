use crate::error::AppError;

pub struct AnalysisResult {
    pub bpm: Option<f64>,
    pub key: Option<String>,
}

pub fn analyze_bpm(_samples: &[f32], _sample_rate: u32) -> Result<Option<f64>, AppError> {
    Err(AppError::NotImplemented("analyze_bpm".to_string()))
}

pub fn analyze_key(_samples: &[f32], _sample_rate: u32) -> Result<Option<String>, AppError> {
    Err(AppError::NotImplemented("analyze_key".to_string()))
}
