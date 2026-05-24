use std::sync::Arc;

use tauri::State;

use crate::db::models::Tag;
use crate::db::pool::DbPool;
use crate::error::AppError;

const TAG_COLS: &str = "id, name, color, is_preset, is_pinned, is_metadata";

fn row_to_tag(row: &rusqlite::Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        is_preset: row.get::<_, i32>(3)? != 0,
        is_pinned: row.get::<_, i32>(4)? != 0,
        is_metadata: row.get::<_, i32>(5)? != 0,
    })
}

fn file_id_from_path(conn: &rusqlite::Connection, path: &str) -> Result<i64, AppError> {
    conn.query_row(
        "SELECT id FROM audio_files WHERE path = ?1",
        rusqlite::params![path],
        |row| row.get(0),
    )
    .map_err(|e| AppError::Database(format!("File not found: {} ({})", path, e)))
}

fn find_or_create_tag(conn: &rusqlite::Connection, name: &str) -> Result<Tag, AppError> {
    if let Ok(tag) = conn.query_row(
        &format!("SELECT {} FROM tags WHERE name = ?1", TAG_COLS),
        rusqlite::params![name],
        row_to_tag,
    ) {
        return Ok(tag);
    }

    conn.execute(
        "INSERT INTO tags (name, color, is_preset, is_pinned, is_metadata) VALUES (?1, NULL, 0, 0, 0)",
        rusqlite::params![name],
    )?;

    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        name: name.to_string(),
        color: None,
        is_preset: false,
        is_pinned: false,
        is_metadata: false,
    })
}

#[tauri::command]
pub fn add_tag(
    pool: State<'_, Arc<DbPool>>,
    file_path: String,
    tag_name: String,
) -> Result<Tag, AppError> {
    let conn = pool.get()?;
    let file_id = file_id_from_path(&conn, &file_path)?;
    let tag = find_or_create_tag(&conn, &tag_name)?;

    conn.execute(
        "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
        rusqlite::params![file_id, tag.id],
    )?;

    Ok(tag)
}

#[tauri::command]
pub fn remove_tag(
    pool: State<'_, Arc<DbPool>>,
    file_path: String,
    tag_id: i64,
) -> Result<(), AppError> {
    let conn = pool.get()?;
    let file_id = file_id_from_path(&conn, &file_path)?;

    conn.execute(
        "DELETE FROM file_tags WHERE file_id = ?1 AND tag_id = ?2",
        rusqlite::params![file_id, tag_id],
    )?;

    Ok(())
}

#[tauri::command]
pub fn list_file_tags(
    pool: State<'_, Arc<DbPool>>,
    file_path: Option<String>,
    exclude_metadata: Option<bool>,
) -> Result<Vec<Tag>, AppError> {
    let conn = pool.get()?;
    let skip_meta = exclude_metadata.unwrap_or(false);

    let tags = if let Some(path) = file_path {
        let file_id = file_id_from_path(&conn, &path)?;
        let sql = if skip_meta {
            format!(
                "SELECT {} FROM tags t JOIN file_tags ft ON t.id = ft.tag_id WHERE ft.file_id = ?1 AND t.is_metadata = 0 ORDER BY t.name",
                TAG_COLS
            )
        } else {
            format!(
                "SELECT {} FROM tags t JOIN file_tags ft ON t.id = ft.tag_id WHERE ft.file_id = ?1 ORDER BY t.name",
                TAG_COLS
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![file_id], row_to_tag)?;
        rows.filter_map(|r| r.ok()).collect()
    } else {
        let sql = format!("SELECT {} FROM tags ORDER BY name", TAG_COLS);
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], row_to_tag)?;
        rows.filter_map(|r| r.ok()).collect()
    };

    Ok(tags)
}

#[tauri::command]
pub fn get_all_tags(
    pool: State<'_, Arc<DbPool>>,
) -> Result<Vec<Tag>, AppError> {
    let conn = pool.get()?;
    let sql = format!("SELECT {} FROM tags ORDER BY name", TAG_COLS);
    let mut stmt = conn.prepare(&sql)?;
    let tags = stmt
        .query_map([], row_to_tag)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tags)
}

#[tauri::command]
pub fn get_pinned_tags(
    pool: State<'_, Arc<DbPool>>,
) -> Result<Vec<Tag>, AppError> {
    let conn = pool.get()?;
    let sql = format!("SELECT {} FROM tags WHERE is_pinned = 1 ORDER BY name", TAG_COLS);
    let mut stmt = conn.prepare(&sql)?;
    let tags = stmt
        .query_map([], row_to_tag)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(tags)
}

#[tauri::command]
pub fn toggle_tag_pin(
    pool: State<'_, Arc<DbPool>>,
    tag_id: i64,
) -> Result<Tag, AppError> {
    let conn = pool.get()?;

    conn.execute(
        "UPDATE tags SET is_pinned = CASE WHEN is_pinned = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        rusqlite::params![tag_id],
    )?;

    let tag = conn.query_row(
        &format!("SELECT {} FROM tags WHERE id = ?1", TAG_COLS),
        rusqlite::params![tag_id],
        row_to_tag,
    )?;

    Ok(tag)
}

#[tauri::command]
pub fn create_tag(
    pool: State<'_, Arc<DbPool>>,
    name: String,
) -> Result<Tag, AppError> {
    let conn = pool.get()?;
    let tag = find_or_create_tag(&conn, &name)?;
    Ok(tag)
}

#[tauri::command]
pub fn delete_tag(
    pool: State<'_, Arc<DbPool>>,
    tag_id: i64,
) -> Result<(), AppError> {
    let conn = pool.get()?;

    let is_preset: bool = conn.query_row(
        "SELECT is_preset FROM tags WHERE id = ?1",
        rusqlite::params![tag_id],
        |row| row.get::<_, i32>(0),
    )
    .map(|v| v != 0)
    .unwrap_or(false);

    if is_preset {
        return Err(AppError::Database("Cannot delete preset tags".to_string()));
    }

    conn.execute("DELETE FROM tags WHERE id = ?1", rusqlite::params![tag_id])?;
    Ok(())
}

#[tauri::command]
pub fn filter_files_by_tag_names(
    pool: State<'_, Arc<DbPool>>,
    tag_names: Vec<String>,
) -> Result<Vec<String>, AppError> {
    if tag_names.is_empty() {
        return Ok(Vec::new());
    }

    let conn = pool.get()?;
    let placeholders: Vec<String> = tag_names.iter().enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    let sql = format!(
        "SELECT f.path FROM audio_files f
         JOIN file_tags ft ON f.id = ft.file_id
         JOIN tags t ON ft.tag_id = t.id
         WHERE t.name IN ({})
         GROUP BY f.id
         HAVING COUNT(DISTINCT t.id) = ?{}",
        placeholders.join(","),
        tag_names.len() + 1
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = tag_names.iter()
        .map(|n| Box::new(n.clone()) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    params.push(Box::new(tag_names.len() as i32));

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let paths = stmt.query_map(param_refs.as_slice(), |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(paths)
}

#[tauri::command]
pub fn get_tag_file_count(
    pool: State<'_, Arc<DbPool>>,
    tag_id: i64,
) -> Result<i64, AppError> {
    let conn = pool.get()?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM file_tags WHERE tag_id = ?1",
        rusqlite::params![tag_id],
        |row| row.get(0),
    )?;
    Ok(count)
}
