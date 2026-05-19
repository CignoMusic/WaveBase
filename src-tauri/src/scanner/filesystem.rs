use crate::error::AppError;

pub struct ScannedFile {
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
    pub modified_at: String,
}

const AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3", "aiff", "aif", "flac", "ogg", "m4a", "wma"];

pub fn is_audio_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn scan_directory(path: &std::path::Path) -> Result<Vec<ScannedFile>, AppError> {
    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(path).follow_links(true) {
        let entry = entry.map_err(|e| {
            AppError::Scan(format!("Failed to walk directory: {}", e))
        })?;

        if entry.file_type().is_file() && is_audio_file(entry.path()) {
            let metadata = entry.metadata().map_err(|e| {
                AppError::Scan(format!("Failed to read metadata: {}", e))
            })?;
            files.push(ScannedFile {
                path: entry.path().to_string_lossy().to_string(),
                filename: entry
                    .file_name()
                    .to_string_lossy()
                    .to_string(),
                extension: entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_lowercase(),
                size_bytes: metadata.len(),
                modified_at: chrono_from_system_time(&metadata.modified().ok())
                    .unwrap_or_default(),
            });
        }
    }

    Ok(files)
}

fn chrono_from_system_time(time: &Option<std::time::SystemTime>) -> Option<String> {
    let time = time.as_ref()?;
    let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
    let secs = duration.as_secs();
    // Format as ISO 8601
    Some(format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        secs / 31536000 + 1970,
        (secs % 31536000) / 2592000 + 1,
        (secs % 2592000) / 86400 + 1,
        (secs % 86400) / 3600,
        (secs % 3600) / 60,
        secs % 60
    ))
}
