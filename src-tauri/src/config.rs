use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        let appdata = std::env::var_os("APPDATA").expect("APPDATA environment variable must be set");
        PathBuf::from(appdata)
    } else if cfg!(target_os = "macos") {
        let home =
            std::env::var_os("HOME").expect("HOME environment variable must be set");
        PathBuf::from(home).join("Library").join("Application Support")
    } else {
        let home =
            std::env::var_os("HOME").expect("HOME environment variable must be set");
        PathBuf::from(home).join(".local").join("share")
    };
    base.join("wavebase")
}

pub fn ensure_data_dir() -> PathBuf {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create app data directory");
    dir
}

pub fn db_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("wavebase.db")
}

pub fn settings_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("settings.toml")
}
