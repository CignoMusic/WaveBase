use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use tauri::State;

use crate::analysis::{decoder, dsp, parser};
use crate::db::models::{ScanProgress, TagProgress};
use crate::db::pool::DbPool;
use crate::error::AppError;
use crate::scanner::filesystem::{self, ScannedFile};

const WORKERS: usize = 4;

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

    // Init progress
    {
        let mut progress = tag_state.progress.lock().unwrap();
        *progress = TagProgress {
            total: file_ids.len(),
            processed: 0,
            status: "scanning".to_string(),
        };
    }

    if !file_ids.is_empty() {
        let pool_clone = Arc::clone(&pool);
        let progress_clone = Arc::clone(&tag_state.progress);
        let count = Arc::new(AtomicUsize::new(0));

        std::thread::spawn(move || {
            let chunk_size = (file_ids.len() + WORKERS - 1) / WORKERS;
            let mut handles = Vec::new();

            for chunk in file_ids.chunks(chunk_size) {
                let chunk_owned: Vec<(i64, String)> = chunk.to_vec();
                let pool_ref = Arc::clone(&pool_clone);
                let count_ref = Arc::clone(&count);

                handles.push(std::thread::spawn(move || {
                    for (file_id, file_path) in &chunk_owned {
                        if let Err(e) = process_single_file(&pool_ref, *file_id, file_path) {
                            eprintln!("Background tagging error for {}: {}", file_path, e);
                        }
                        count_ref.fetch_add(1, Ordering::Relaxed);
                    }
                }));
            }

            // Progress updater thread
            let progress_up = Arc::clone(&progress_clone);
            let count_up = Arc::clone(&count);
            let total = file_ids.len();
            std::thread::spawn(move || {
                loop {
                    let done = count_up.load(Ordering::Relaxed);
                    {
                        let mut p = progress_up.lock().unwrap();
                        p.processed = done;
                    }
                    if done >= total {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
                let mut p = progress_up.lock().unwrap();
                p.processed = total;
                p.status = "complete".to_string();
            });

            for h in handles {
                let _ = h.join();
            }
        });
    } else {
        let mut p = tag_state.progress.lock().unwrap();
        p.status = "complete".to_string();
    }

    Ok(files)
}

fn process_single_file(
    pool: &Arc<DbPool>,
    file_id: i64,
    file_path: &str,
) -> Result<(), AppError> {
    let conn = pool.get()?;

    // Skip files that already have all metadata from a previous scan
    let already_analyzed: bool = conn.query_row(
        "SELECT bpm IS NOT NULL AND key IS NOT NULL FROM audio_files WHERE id = ?1",
        rusqlite::params![file_id],
        |row| row.get(0),
    ).unwrap_or(false);

    let filename: String = conn.query_row(
        "SELECT filename FROM audio_files WHERE id = ?1",
        rusqlite::params![file_id],
        |row| row.get(0),
    )?;

    // Always parse filename (instant, gives us track_type even for re-scanned files)
    let parsed = parser::parse_filename(&filename);

    let needs_bpm_analysis = !already_analyzed && parsed.bpm.is_none();
    let needs_key_analysis = !already_analyzed && parsed.key.is_none();

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

    let final_bpm = if already_analyzed {
        None // don't overwrite existing DB value
    } else {
        parsed.bpm.or_else(|| analyzed_bpm.map(|b| b as i32))
    };

    let final_key = if already_analyzed {
        None
    } else {
        parsed.key.clone().or(analyzed_key.clone())
    };

    let bpm_analyzed = !already_analyzed && parsed.bpm.is_none() && analyzed_bpm.is_some();
    let key_analyzed = !already_analyzed && parsed.key.is_none() && analyzed_key.is_some();

    // Store artists as comma-separated in the artist field
    let artist_str = if parsed.artists.is_empty() {
        None
    } else {
        Some(parsed.artists.join(", "))
    };

    if !already_analyzed {
        conn.execute(
            "UPDATE audio_files SET track_name = ?1, bpm = COALESCE(?2, bpm), key = COALESCE(?3, key),
             artist = ?4, bpm_analyzed = ?5, key_analyzed = ?6, updated_at = datetime('now')
             WHERE id = ?7",
            rusqlite::params![
                parsed.track_name,
                final_bpm,
                final_key,
                artist_str,
                bpm_analyzed as i32,
                key_analyzed as i32,
                file_id,
            ],
        )?;
    }

    // Create and link tags (always run, ensures tags exist even for re-scanned files)
    // Artist tags (non-metadata, pinned, user-managed)
    for artist_name in &parsed.artists {
        let tag_name = format!("@{}", artist_name);
        let tag = find_or_create_tag_inner(&conn, &tag_name, false)?;
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![file_id, tag.id],
        )?;
    }

    // BPM tag (metadata, hidden from tags column)
    if let Some(bpm) = final_bpm.or_else(|| {
        // For already-analyzed files, read the stored BPM
        conn.query_row(
            "SELECT bpm FROM audio_files WHERE id = ?1",
            rusqlite::params![file_id],
            |row| row.get::<_, Option<i32>>(0),
        ).ok().flatten()
    }) {
        let tag_name = format!("{} BPM", bpm);
        let tag = find_or_create_tag_inner(&conn, &tag_name, true)?;
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![file_id, tag.id],
        )?;
    }

    // Key tag (metadata, hidden from tags column)
    if let Some(ref key) = final_key.or_else(|| {
        conn.query_row(
            "SELECT key FROM audio_files WHERE id = ?1",
            rusqlite::params![file_id],
            |row| row.get::<_, Option<String>>(0),
        ).ok().flatten()
    }) {
        let tag = find_or_create_tag_inner(&conn, key, true)?;
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![file_id, tag.id],
        )?;
    }

    // Track type tag (pinned, non-metadata)
    if let Some(ref track_type) = parsed.track_type {
        let tag = find_or_create_tag_inner(&conn, track_type, false)?;
        if !tag.is_pinned {
            conn.execute(
                "UPDATE tags SET is_pinned = 1 WHERE id = ?1",
                rusqlite::params![tag.id],
            )?;
        }
        conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![file_id, tag.id],
        )?;
    }

    // Duration-based time code tag (non-metadata)
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
            let tag = find_or_create_tag_inner(&conn, &time_tag, false)?;
            conn.execute(
                "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![file_id, tag.id],
            )?;
        }
    }

    Ok(())
}

fn find_or_create_tag_inner(conn: &rusqlite::Connection, name: &str, is_metadata: bool) -> Result<crate::db::models::Tag, AppError> {
    use crate::db::models::Tag;

    if let Ok(tag) = conn.query_row(
        "SELECT id, name, color, is_preset, is_pinned, is_metadata FROM tags WHERE name = ?1",
        rusqlite::params![name],
        |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                is_preset: row.get::<_, i32>(3)? != 0,
                is_pinned: row.get::<_, i32>(4)? != 0,
                is_metadata: row.get::<_, i32>(5)? != 0,
            })
        },
    ) {
        return Ok(tag);
    }

    conn.execute(
        "INSERT INTO tags (name, color, is_preset, is_pinned, is_metadata) VALUES (?1, NULL, 0, 0, ?2)",
        rusqlite::params![name, is_metadata as i32],
    )?;

    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        name: name.to_string(),
        color: None,
        is_preset: false,
        is_pinned: false,
        is_metadata,
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
