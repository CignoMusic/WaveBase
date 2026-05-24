use std::sync::{Arc, Mutex};

use tauri::State;

use crate::analysis::{decoder, dsp, parser};
use crate::db::models::{ScanProgress, TagProgress};
use crate::db::pool::DbPool;
use crate::error::AppError;
use crate::scanner::filesystem::{self, ScannedFile};

pub struct BackgroundTagState {
    pub progress: Arc<Mutex<TagProgress>>,
}

#[tauri::command]
pub fn scan_directory(
    pool: State<'_, Arc<DbPool>>,
    tag_state: State<'_, Arc<BackgroundTagState>>,
    path: String,
) -> Result<Vec<ScannedFile>, AppError> {
    let dir = std::path::Path::new(&path);
    let files = filesystem::scan_directory(dir)?;

    let conn = pool.get()?;
    let mut file_ids: Vec<(i64, String)> = Vec::new();

    for file in &files {
        let folder_path = std::path::Path::new(&file.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        conn.execute(
            "INSERT OR IGNORE INTO audio_files (path, filename, folder_path, extension, size_bytes, modified_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![file.path, file.filename, folder_path, file.extension, file.size_bytes as i64, file.modified_at],
        )?;

        let id: i64 = conn.query_row(
            "SELECT id FROM audio_files WHERE path = ?1",
            rusqlite::params![file.path],
            |row| row.get(0),
        )?;
        file_ids.push((id, file.path.clone()));
    }

    // Initialize progress and spawn background tagging
    let total = file_ids.len();
    {
        let mut progress = tag_state.progress.lock().unwrap();
        *progress = TagProgress {
            total,
            processed: 0,
            status: "scanning".to_string(),
        };
    }

    let pool_clone: Arc<DbPool> = Arc::clone(&pool);
    let progress_clone = Arc::clone(&tag_state.progress);
    let paths: Vec<(i64, String)> = file_ids.clone();

    std::thread::spawn(move || {
        for (file_id, file_path) in &paths {
            if let Err(e) = process_single_file(
                &pool_clone,
                *file_id,
                file_path,
            ) {
                eprintln!("Background tagging error for {}: {}", file_path, e);
            }

            let mut progress = progress_clone.lock().unwrap();
            progress.processed += 1;
        }

        let mut progress = progress_clone.lock().unwrap();
        progress.status = "complete".to_string();
    });

    Ok(files)
}

fn process_single_file(
    pool: &Arc<DbPool>,
    file_id: i64,
    file_path: &str,
) -> Result<(), AppError> {
    let conn = pool.get()?;

    // Get filename for parsing
    let filename: String = conn.query_row(
        "SELECT filename FROM audio_files WHERE id = ?1",
        rusqlite::params![file_id],
        |row| row.get(0),
    )?;

    // Parse filename
    let parsed = parser::parse_filename(&filename);

    // Determine which fields need analysis
    let needs_bpm_analysis = parsed.bpm.is_none();
    let needs_key_analysis = parsed.key.is_none();

    // Run audio analysis if needed
    let (analyzed_bpm, analyzed_key) = if needs_bpm_analysis || needs_key_analysis {
        match decoder::decode_audio_to_mono(file_path) {
            Ok((samples, sample_rate)) => {
                let abpm = if needs_bpm_analysis {
                    dsp::analyze_bpm(&samples, sample_rate).unwrap_or(None)
                } else {
                    None
                };
                let akey = if needs_key_analysis {
                    dsp::analyze_key(&samples, sample_rate).unwrap_or(None)
                } else {
                    None
                };
                (abpm, akey)
            }
            Err(e) => {
                eprintln!("Audio decode failed for {}: {}", file_path, e);
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    let final_bpm = parsed.bpm.or_else(|| analyzed_bpm.map(|b| b as i32));
    let final_key = parsed.key.clone().or(analyzed_key.clone());
    let bpm_analyzed = parsed.bpm.is_none() && analyzed_bpm.is_some();
    let key_analyzed = parsed.key.is_none() && analyzed_key.is_some();

    // Update DB
    conn.execute(
        "UPDATE audio_files SET track_name = ?1, bpm = ?2, key = ?3, artist = ?4,
         bpm_analyzed = ?5, key_analyzed = ?6, updated_at = datetime('now')
         WHERE id = ?7",
        rusqlite::params![
            parsed.track_name,
            final_bpm,
            final_key,
            parsed.artist,
            bpm_analyzed as i32,
            key_analyzed as i32,
            file_id,
        ],
    )?;

    // Auto-create tags from parsed/analyzed values
    if let Some(ref artist) = parsed.artist {
        let tag = find_or_create_tag_inner(&conn, &format!("@{}", artist))?;
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![file_id, tag.id],
        )?;
    }

    // BPM tag
    if let Some(bpm) = final_bpm {
        let tag = find_or_create_tag_inner(&conn, &format!("{} BPM", bpm))?;
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![file_id, tag.id],
        )?;
    }

    // Key tag
    if let Some(ref key) = final_key {
        let tag = find_or_create_tag_inner(&conn, key)?;
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![file_id, tag.id],
        )?;
    }

    // Track type tag
    if let Some(ref track_type) = parsed.track_type {
        let tag = find_or_create_tag_inner(&conn, track_type)?;
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![file_id, tag.id],
        )?;
    }

    // Duration-based time code tag
    let duration: Option<f64> = conn.query_row(
        "SELECT duration_secs FROM audio_files WHERE id = ?1",
        rusqlite::params![file_id],
        |row| row.get(0),
    ).ok().flatten();

    if let Some(secs) = duration {
        if secs > 0.0 {
            let mins = (secs / 60.0).floor() as i32;
            let secs_remain = secs as i32 % 60;
            let time_tag = format!("{}:{:02}", mins, secs_remain);
            let tag = find_or_create_tag_inner(&conn, &time_tag)?;
            conn.execute(
                "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![file_id, tag.id],
            )?;
        }
    }

    Ok(())
}

fn find_or_create_tag_inner(conn: &rusqlite::Connection, name: &str) -> Result<crate::db::models::Tag, AppError> {
    if let Ok(tag) = conn.query_row(
        "SELECT id, name, color, is_preset FROM tags WHERE name = ?1",
        rusqlite::params![name],
        |row| {
            Ok(crate::db::models::Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                is_preset: row.get::<_, i32>(3)? != 0,
            })
        },
    ) {
        return Ok(tag);
    }

    conn.execute(
        "INSERT INTO tags (name, color, is_preset) VALUES (?1, NULL, 0)",
        rusqlite::params![name],
    )?;

    let id = conn.last_insert_rowid();
    Ok(crate::db::models::Tag {
        id,
        name: name.to_string(),
        color: None,
        is_preset: false,
    })
}

#[tauri::command]
pub fn get_tag_progress(
    tag_state: State<'_, Arc<BackgroundTagState>>,
) -> Result<TagProgress, AppError> {
    let progress = tag_state.progress.lock().unwrap();
    Ok(progress.clone())
}

#[tauri::command]
pub fn scan_status() -> Result<ScanProgress, AppError> {
    Err(AppError::NotImplemented("scan_status".to_string()))
}
