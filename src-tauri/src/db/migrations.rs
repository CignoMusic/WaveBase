use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS audio_files (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            path        TEXT NOT NULL UNIQUE,
            filename    TEXT NOT NULL,
            folder_path TEXT NOT NULL,
            extension   TEXT NOT NULL,
            size_bytes  INTEGER NOT NULL DEFAULT 0,
            modified_at TEXT NOT NULL,
            duration_secs REAL,
            track_name  TEXT,
            bpm         INTEGER,
            key         TEXT,
            artist      TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_audio_files_path ON audio_files(path);
        CREATE INDEX IF NOT EXISTS idx_audio_files_folder ON audio_files(folder_path);
        CREATE INDEX IF NOT EXISTS idx_audio_files_bpm ON audio_files(bpm);
        CREATE INDEX IF NOT EXISTS idx_audio_files_key ON audio_files(key);
        CREATE INDEX IF NOT EXISTS idx_audio_files_artist ON audio_files(artist);

        CREATE TABLE IF NOT EXISTS tags (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT NOT NULL UNIQUE,
            color TEXT
        );

        CREATE TABLE IF NOT EXISTS file_tags (
            file_id INTEGER NOT NULL,
            tag_id  INTEGER NOT NULL,
            PRIMARY KEY (file_id, tag_id),
            FOREIGN KEY (file_id) REFERENCES audio_files(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_file_tags_file_id ON file_tags(file_id);
        CREATE INDEX IF NOT EXISTS idx_file_tags_tag_id ON file_tags(tag_id);

        CREATE TABLE IF NOT EXISTS scan_roots (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE
        );
        ",
    )?;
    Ok(())
}
