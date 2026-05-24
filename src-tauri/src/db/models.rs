use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: i64,
    pub path: String,
    pub filename: String,
    pub folder_path: String,
    pub extension: String,
    pub size_bytes: i64,
    pub modified_at: String,
    pub duration_secs: Option<f64>,
    pub track_name: Option<String>,
    pub bpm: Option<i32>,
    pub key: Option<String>,
    pub artist: Option<String>,
    pub bpm_analyzed: bool,
    pub key_analyzed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub is_preset: bool,
    pub is_pinned: bool,
    pub is_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagProgress {
    pub total: usize,
    pub processed: usize,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTag {
    pub file_id: i64,
    pub tag_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRoot {
    pub id: i64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total: usize,
    pub scanned: usize,
    pub new_files: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMetadata {
    pub track_name: Option<String>,
    pub bpm: Option<i32>,
    pub key: Option<String>,
    pub artists: Vec<String>,
    pub track_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFileWithTags {
    pub file: AudioFile,
    pub tags: Vec<Tag>,
}
