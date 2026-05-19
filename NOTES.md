# WaveBase — Session Notes

## Status (after Session 2)
- Frontend UI shell built (Titlebar, Toolbar, TrackList, PlayerBar)
- Mock track data rendered with stem expansion
- Waveform canvas with synthetic data, play/pause/seek working
- `src/lib/ui-logic.ts` created with stubs for future frontend logic

## Known issues / Next session priorities

### 1. Auto-play on startup (FIXED)
Player no longer auto-plays when a track is selected.

### 2. Scan Directory — wire to Tauri dialog
Currently uses `<input type="file" webkitdirectory>` which has limitations:
- Uses browser file upload semantics instead of native folder picker
- Can't get the real filesystem path in all cases

**Fix**: Use `@tauri-apps/plugin-dialog` to open a native directory picker and pass the path to the Rust `scan_directory` command.

### 3. Replace mock data with real database queries
- Wire `TrackList` to the Rust `list_files` / `search_files` commands
- Wire `Toolbar` filter pills and search to SQLite queries
- Wire `PlayerBar` to real file data (duration, BPM, key from DB)

### 4. Audio playback
- Connect play/pause/seek to Rodio via Tauri commands
- Replace synthetic waveform with real decoded audio samples
- Replace static `154s` duration with per-file duration from the database

### 5. Filename parser
- Implement the smart filename parser in `analysis/parser.rs`
- Auto-extract BPM, key, artist from filenames during scanning

### 6. Audio analysis
- Wire Symphonia decoding + stratum-dsp analysis as fallback
- Only analyze fields that weren't parsed from the filename

### 7. Other UI polish
- Traffic lights / window controls removed (intentional)
- New & Upload buttons removed (intentional)
- Filter and search are visual-only — not functional yet
