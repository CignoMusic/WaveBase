use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub bpm: Option<f64>,
    pub key: Option<String>,
}

pub fn analyze_bpm(samples: &[f32], sample_rate: u32) -> Result<Option<f64>, AppError> {
    let result = stratum_dsp::analyze_audio(samples, sample_rate, stratum_dsp::AnalysisConfig::default())
        .map_err(|e| AppError::Analysis(format!("stratum-dsp analysis failed: {}", e)))?;

    if result.bpm_confidence > 0.3 {
        Ok(Some(result.bpm as f64))
    } else {
        Ok(None)
    }
}

pub fn analyze_key(samples: &[f32], sample_rate: u32) -> Result<Option<String>, AppError> {
    let result = stratum_dsp::analyze_audio(samples, sample_rate, stratum_dsp::AnalysisConfig::default())
        .map_err(|e| AppError::Analysis(format!("stratum-dsp analysis failed: {}", e)))?;

    if result.key_confidence > 0.3 {
        Ok(Some(result.key.name().to_string()))
    } else {
        Ok(None)
    }
}
