# WaveBase — Project Information & Session Guide

---

## 1. Session Protocol — MUST READ FIRST

### For the AI agent:
You **must** update this file at the end of every session. After completing any feature, fix, or significant change:

1. Update the status indicators (✅ ⬜ ⚠️) in Architecture and Feature Roadmap sections
2. Add a new entry to the Session Log with what was accomplished
3. Update Session Priority to reflect what should be worked on next
4. Add any new known issues or decisions to the appropriate sections
5. Always leave the file in a state that accurately reflects the project

### For the human:
Start every new session by saying:

> **"Follow the instructions in PROJECT-INFO.md"**

This tells the AI to read this file first to understand the project's current state and what to work on next.

---

## 2. Project Overview

**WaveBase** is a fast, lightweight audio library manager for music producers.

Music producers have thousands of audio files spread across their hard drives — loops, one-shots, stems, project exports, sample packs. WaveBase lets them point the app at one or more directories, scans and indexes all audio files, and presents everything in a clean, fast, well-organised interface. Producers can tag, search, preview and manage their entire library without ever leaving the app.

---

## 3. Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| App shell | Tauri 2 | Cross-platform desktop, system integration, file access |
| Backend | Rust | File scanning, metadata parsing, DB, audio analysis, playback |
| Frontend | React 19 + TypeScript | All UI |
| Build tool | Vite 6 | Dev server, bundling |
| Database | SQLite via rusqlite (r2d2 pool) | Local single-file library storage |
| Audio decode | Symphonia 0.5 | Read/decode WAV, MP3, AIFF, FLAC, OGG, M4A |
| Audio playback | Rodio 0.17 | Playback via Rust backend |
| BPM/Key analysis | stratum-dsp 1 | Audio analysis fallback |
| Waveform UI | Canvas 2D (real peak data via Symphonia) | Waveform rendering in frontend |
| Styling | Tailwind CSS 3 + custom CSS variables | Dark theme, DAW-like UI |
| Dialog | @tauri-apps/plugin-dialog 2 | Native folder picker |
| File watching | notify 6 | Real-time filesystem monitoring |

### Key Rust Crates

| Crate | Version | Use |
|-------|---------|-----|
| `rusqlite` | 0.31 (bundled) | SQLite with bundled sqlite3 |
| `r2d2` / `r2d2_sqlite` | 0.8 / 0.24 | Connection pooling |
| `symphonia` | 0.5 (all features) | Audio decoding |
| `rodio` | 0.17 (symphonia-all) | Audio playback |
| `stratum-dsp` | 1 | BPM + musical key detection |
| `notify` | 6 | File system watcher |
| `walkdir` | 2 | Recursive directory walking |
| `serde` / `serde_json` | 1 | Serialization |
| `thiserror` | 1 | Error handling |
| `tokio` | 1 | Async runtime |

---

## 4. Architecture

### Backend Modules

```
src-tauri/src/
├── main.rs              # Entry point, calls wavebase_lib::run()
├── lib.rs               # Tauri builder, manages state, registers commands
├── config.rs            # Data directory, DB path, settings path
├── error.rs             # AppError enum (Database, Io, Playback, Scan, Analysis, etc.)
│
├── commands/            # Tauri command handlers (invoke targets)
│   ├── mod.rs           # Module declarations
│   ├── scan.rs          # scan_directory, scan_status
│   ├── library.rs       # search_files, get_file, list_files
│   ├── playback.rs      # play_audio, toggle_playback, pause_audio, resume_audio, stop_audio, get_playback_status, set_volume, set_duration, store_track_duration, get_waveform_data
│   └── tags.rs          # add_tag, remove_tag, list_tags
│
├── db/                  # Database layer
│   ├── mod.rs           # Module declarations
│   ├── models.rs        # AudioFile, Tag, FileTag, ScanRoot, ScanProgress, ParsedMetadata
│   ├── migrations.rs    # Schema: audio_files, tags, file_tags, scan_roots + indexes
│   └── pool.rs          # r2d2 SqliteConnectionManager pool (max 4 connections)
│
├── scanner/             # File system scanning
│   ├── mod.rs           # Module declarations
│   ├── filesystem.rs    # walkdir-based recursive scan, audio extension filter
│   └── watcher.rs       # notify-based file watcher (stub)
│
├── playback/            # Audio playback
│   ├── mod.rs           # Module declarations
│   └── player.rs        # AudioPlayer with Rodio — play, pause, resume, stop, toggle, volume, status
│
└── analysis/            # Audio metadata analysis
    ├── mod.rs           # Module declarations
    ├── parser.rs        # Smart filename parsing (stub with test skeleton)
    ├── decoder.rs       # Symphonia audio decoding (stub)
    └── dsp.rs           # stratum-dsp BPM/key analysis (stub)
```

### Frontend Components

```
src/
├── main.tsx             # React entry point
├── App.tsx              # Root component — orchestrates scanning, selection, state
├── index.css            # All styles (Tailwind directives + full custom dark theme)
├── vite-env.d.ts        # Vite client types
│
├── components/
│   ├── Titlebar.tsx     # Top bar: app name + "Scan Directory" button (uses @tauri-apps/plugin-dialog)
│   ├── Toolbar.tsx      # Filter pills (All/Beats/Loops/Stems) + search input
│   ├── TrackList.tsx    # Scrollable track list with collapsible stems, selection, drag handles
│   └── PlayerBar.tsx    # Bottom player: transport controls, waveform canvas, progress, volume
│
└── lib/
    └── ui-logic.ts      # Utilities: formatTime, waveform generation/rendering, state factory stubs
```

### Data Flow

```
User clicks "Scan Directory"
  → @tauri-apps/plugin-dialog opens native folder picker
  → invoke('scan_directory', { path }) to Rust backend
  → walkdir scans for audio files
  → Files inserted into SQLite audio_files table
  → Vec<ScannedFile> returned to frontend
  → Frontend maps to Track[] and renders in TrackList
        ↓
User clicks a track
  → Frontend selects it, PlayerBar updates
  → TODO: invoke('play_audio', { path }) to Rust backend
  → TODO: Rodio plays audio
  → TODO: Real waveform data from decoded audio samples
        ↓
User types in search box
  → TODO: Debounced invoke('search_files', { query })
  → TODO: SQLite query with proper indexing
  → TODO: Results rendered in TrackList
```

### Database Schema

```sql
audio_files (
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
-- Indexes: path, folder_path, bpm, key, artist

tags (
  id    INTEGER PRIMARY KEY AUTOINCREMENT,
  name  TEXT NOT NULL UNIQUE,
  color TEXT
);

file_tags (
  file_id INTEGER NOT NULL,
  tag_id  INTEGER NOT NULL,
  PRIMARY KEY (file_id, tag_id),
  FOREIGN KEY → audio_files(id) ON DELETE CASCADE,
  FOREIGN KEY → tags(id) ON DELETE CASCADE
);
-- Indexes: file_id, tag_id

scan_roots (
  id   INTEGER PRIMARY KEY AUTOINCREMENT,
  path TEXT NOT NULL UNIQUE
);
```

---

## 5. Current State (Session 6)

### ✅ Working / Stable
- App launches, shows full UI
- "Scan Directory" picks a folder, scans for audio files, returns results
- Files inserted into SQLite `audio_files` table
- UI renders: Titlebar, Toolbar with filters, TrackList with mock data, PlayerBar with waveform canvas
- Selection state, stem expansion work
- Database connection pool, migrations, schema all set up
- Custom dark theme CSS complete (no Tailwind utility classes used yet)
- Clean app data dir management (%APPDATA%/wavebase, ~/Library/Application Support/wavebase)
- **Real audio playback via Rodio** — play, pause, resume, stop, volume control all working
- **Playback position & duration tracking** — position counter, playhead, and progress fill all update in real time
- **Track `path` field** — added to Track interface, used for playback targeting
- **Next/prev track navigation** — prev restarts track first click, then goes to previous; next wraps around
- **Smart prev button** — first click restarts current track, second click goes to previous
- **Real waveform from audio data** — waveform peaks extracted via Symphonia on the Rust backend, replaces synthetic sine waves
- **DB-stored track duration** — real duration saved to `audio_files.duration_secs` when waveform decode completes; used on subsequent plays for instant accurate progress

### ⚠️ Partial / Needs Wiring
| Component | Issue | Details |
|-----------|-------|---------|
| `App.tsx` → TrackList | Uses mock data | Scanned files are returned but frontend doesn't query DB — maps `invoke<ScannedFile[]>` response directly to `Track[]`, doesn't use `list_files` |
| `lib/ui-logic.ts` | Stubs not wired | Playback, search, scan, library, tags state factories exist but aren't connected to Tauri commands |
| `Toolbar.tsx` | Visual only | Filter pills and search input render but don't do anything |

### ⬜ Not Yet Implemented (Stubs)
| Module | Files | What's Missing |
|--------|-------|----------------|
| **Filename parser** | `analysis/parser.rs` | Extract BPM, key, artist from filename patterns |
| **Audio analysis** | `analysis/decoder.rs`, `analysis/dsp.rs` | Symphonia decoding + stratum-dsp BPM/key detection |
| **Library commands** | `commands/library.rs` | search_files, get_file, list_file |
| **Tag commands** | `commands/tags.rs` | add_tag, remove_tag, list_tags |
| **File watcher** | `scanner/watcher.rs` | Real-time filesystem monitoring |
| **Bundle detection** | Not started | Group related files by folder |
| **Settings** | `config.rs` has path | No TOML read/write yet |

---

## 6. Complete Feature Roadmap

### Feature 1: Directory Scanning ✅ (Working)
- [x] User picks folder via native dialog
- [x] Recursive scan for audio files (wav, mp3, aiff, aif, flac, ogg, m4a, wma)
- [x] Files inserted into SQLite (INSERT OR IGNORE)
- [x] Non-blocking (though currently synchronous in command)
- [ ] Incremental scanning (track modified_at, only scan new/changed files)
- [ ] Background scanning with progress reporting (scan_status command)
- [ ] Multiple root directories
- [ ] File watcher for real-time updates

### Feature 2: Smart Filename Parsing & Auto-Tagging ⬜
- [ ] Parse filename for BPM (pattern: `123bpm`, `123 BPM`, case-insensitive)
- [ ] Parse filename for key (pattern: `D Minor`, `Dmin`, `Dm`, `D#maj`, normalize to full notation)
- [ ] Parse filename for artist (pattern: `@username`)
- [ ] Extract track name (remaining text after removing structured tokens)
- [ ] Treat underscores/dashes as spaces
- [ ] Case-insensitive matching for BPM and key
- [ ] Tokens can appear in any position in filename
- [ ] Store parsed values in DB fields (bpm, key, artist, track_name)
- [ ] Parsed values automatically become editable tags

### Feature 3: Audio Analysis Fallback ⬜
- [ ] Decode audio with Symphonia (WAV, MP3, AIFF, FLAC, OGG, M4A)
- [ ] BPM detection via stratum-dsp
- [ ] Key detection via stratum-dsp
- [ ] Only analyze fields not already parsed from filename
- [ ] Skip analysis if filename already provided the value (keep scanning fast)
- [ ] Handle analysis errors gracefully (leave field empty for manual tagging)

### Feature 4: Audio Preview ✅ (Position tracking fixed)
- [x] Play audio via Rodio triggered by Tauri commands
- [x] Play/pause/resume/stop controls wired to real playback
- [x] Volume control (Rodio Sink::set_volume)
- [x] Position polling from backend every 200ms
- [x] Backend tracks position accounting for pauses
- [x] Auto-detect when playback finishes (Sink::empty)
- [x] Real waveform from audio data (Symphonia peak extraction, canvas rendering)
- [x] Per-file duration from decoded audio or Symphonia probe, stored in DB for instant use on subsequent plays
- [ ] Click-to-seek on waveform (Rodio Sink doesn't support seeking — source recreation needed)

### Feature 5: Advanced Manual Tagging ⬜
- [ ] Add/remove/list tags per file
- [ ] Tag colors
- [ ] Tag management (rename, delete, merge)
- [ ] Multi-tag filtering
- [ ] Auto-detected values stored as editable tags

### Feature 6: Bundle Detection ⬜
- [ ] Detect related files sharing a parent folder
- [ ] Collapsible bundle UI element
- [ ] Expand bundle to see individual files
- [ ] Interact with bundle as a unit
- [ ] Signals: folder structure + naming conventions
- [ ] Not limited to any genre or file type

### Feature 7: Open in File Explorer ⬜
- [ ] Right-click context menu on file/bundle
- [ ] Open location in OS file explorer
- [ ] Windows Explorer / Mac Finder support

### Feature 8: Fast Startup & Search ⬜
- [ ] Real-time search as user types
- [ ] Search across: filename, track_name, tags, BPM, key, artist, folder_path
- [ ] SQLite queries with proper indexing
- [ ] Instant results even with 50,000+ files
- [ ] App opens and is fully interactive with minimal delay

### Feature 9: Clean Minimal UI ⚠️ (Skeleton built, needs wiring)
- [x] Sidebar/toolbar navigation
- [x] File list with columns (Name, BPM, Key, Artists)
- [x] Waveform panel on selection (real audio data via Symphonia)
- [x] Dark mode (default)
- [ ] Light mode option
- [ ] DAW-like, familiar to producers
- [ ] Wire all visual elements to real data/commands

### Feature 10: Extensibility ⬜
- [ ] Clean modular data layer
- [ ] Future: Google Sheets sync
- [ ] Future: CSV export
- [ ] Future: Cloud storage connectors
- [ ] Future: Other producer workflow integrations

### Hard Requirements (Clean Install/Uninstall) ⬜
- [ ] All user data in single OS-appropriate directory only
- [ ] Settings as human-readable TOML
- [ ] Uninstaller option to delete data directory entirely
- [ ] No registry entries, no background services, no auto-start
- [ ] Min binary size, avoid unnecessary bloat
- [ ] Reinstall with existing data dir = pick up where you left off
- [ ] Fresh install after clean uninstall = start fresh

---

## 7. Implementation Details & Decisions

### Data Directory
- **Windows:** `%APPDATA%\wavebase`
- **macOS:** `~/Library/Application Support/wavebase`
- **Linux:** `~/.local/share/wavebase`
- Database: `wavebase.db`
- Settings: `settings.toml` (human-readable TOML)
- No files may be written anywhere else on the system.

### CSS Architecture
- Uses CSS custom properties (variables) for theming: `--bg`, `--surface`, `--text`, etc.
- Dark theme is default and only theme implemented
- No Tailwind utility classes used in component files — all styling via `index.css` class names
- DAW-inspired design: dark backgrounds, monospace for BPM/key, colored dots for file type
- Grid layout for track rows: `32px 28px 1fr 72px 96px 1fr` (drag-handle, dot, name, bpm, key, artists)

### Audio Formats Supported
`wav`, `mp3`, `aiff`, `aif`, `flac`, `ogg`, `m4a`, `wma`

### Database Conventions
- Connection pool: r2d2 with max 4 connections
- `INSERT OR IGNORE` for file insertion (path is UNIQUE)
- Proper indexes on frequently queried columns
- `ON DELETE CASCADE` on file_tags
- Timestamps as ISO 8601 strings via SQLite `datetime('now')`

### Naming Conventions
- Rust: snake_case for functions/variables, CamelCase for types
- TypeScript: camelCase for functions/variables, PascalCase for types/components
- CSS: kebab-case for class names
- Tables: snake_case (audio_files, file_tags, scan_roots)
- Directories: lowercase (src/components, src/lib, src-tauri/src/db)

### Git Branch Strategy
- `main` — stable, merged branches
- Feature branches: `Directory-Scanning`, `Simple-interface-skeleton`, etc.
- PRs merged into main

---

## 8. Build & Run Commands

```bash
# Development
run.bat                    # npm run tauri dev
npm run tauri dev          # Start dev server + Tauri window

# Building
npm run build              # tsc + vite build
npm run tauri build        # Production build

# Frontend only
npm run dev                # Vite dev server (no Tauri window)
npm run preview            # Preview production build

# Project management
npm run tauri              # Tauri CLI
```

---

## 9. Session Log

### Session 1 — Initial Scaffolding
- Entire project structure scaffolded (Tauri 2 + React + Vite + SQLite + all crates)
- Database schema, models, migrations, pool set up
- All module stubs created (commands, scanner, playback, analysis)
- Backend compiles, app runs with "WaveBase" text

### Session 2 — UI Skeleton + Directory Scanning
- Full UI built: Titlebar, Toolbar, TrackList, PlayerBar
- Custom dark theme CSS (DAW-like design)
- Waveform canvas with synthetic data, play/pause/seek animation
- `ui-logic.ts` with utility functions and state stubs
- Directory scanning wired (native folder picker + walkdir scan)
- Files inserted into SQLite on scan
- Stem expansion UI, selection state, drag handles
- NOTES.md created with known issues and next priorities
- `PROJECT-INFO.md` created as the new master guide (NOTES.md removed)

### Session 3 — Real Audio Playback (Rodio)
- `playback/player.rs` completely rewritten: `AudioPlayer` wraps Rodio `OutputStream` + `Sink`
- Position tracking with pause-aware logic (accounts for cumulative pause duration)
- 7 Tauri commands: `play_audio`, `toggle_playback`, `pause_audio`, `resume_audio`, `stop_audio`, `get_playback_status`, `set_volume`
- `AudioPlayer` managed as Tauri state, initialized during `setup`
- Frontend `PlayerBar.tsx` rewired: removed synthetic RAF loop, added polling via `get_playback_status` every 200ms
- Play/pause/stop buttons call real Tauri commands
- Volume slider wired to `set_volume` command
- `Track` interface gained `path` field for playback targeting
- `App.tsx` maps `path` from scanned files

### Session 4 — Playback Position Fix + Nav Controls
- Added next/prev track navigation buttons with wrap-around
- Smart prev button: first click restarts current track, second click goes to previous track
- Fixed position not updating bug: `pos.max(0.0).min(0.0)` was clamping position to zero when `source.total_duration()` returns `None` (common for MP3). Changed to `min(if duration > 0.0 { duration } else { f64::MAX })`
- Fixed frontend time counter to omit ` / 0:00` when duration is unknown
- Removed debug overlay after confirming position tracking works
- **Fixed playhead not moving** — added Symphonia probe fallback (`probe_duration`) in `player.rs:140` to determine audio file duration when Rodio's `total_duration()` returns `None`
- Added frontend safety net (`maxPositionRef`) for the edge case where even Symphonia probing fails

### Session 6 — Progress Bar No Longer Guesses (DB-Stored Duration)
- **Stored real duration in DB after waveform decode** — Added `set_stored_duration()` helper in `commands/playback.rs` that writes to `audio_files.duration_secs`
- **Read stored duration from DB before playback** — `play_audio` and `toggle_playback` now query the DB for a stored duration before falling back to file-size estimate. If found, `player.set_duration()` immediately corrects the backend state
- **Added `store_track_duration` Tauri command** — called from the frontend's waveform `.then()` handler alongside the existing `set_duration` invoke
- **Result:** Second play of any MP3 file shows accurate progress from second one. No more guessing

### Session 5 — Real Waveform from Audio Data
- Created `playback/waveform.rs` — `compute_waveform_peaks()` uses Symphonia to decode audio and extract peak amplitude per time window
- `WaveformData` struct returns both `peaks: Vec<f64>` and `duration: f64` (computed from `codec_params.time_base` + `n_frames`)
- Added `get_waveform_data` Tauri command returning `WaveformData`
- Updated `PlayerBar.tsx` to call `get_waveform_data` only when playback starts (not on track selection), cache result in `waveRef`
- Duration from waveform decode updates frontend `status.duration` once available (replaces 0:00 for MP3 files)
- **Fixed 5-second playback delay** — removed Symphonia `probe_duration` from Play/Toggle commands (reverted to `unwrap_or(0.0)`). Symphonia's MP3 format reader was scanning the entire file to build a seek index, blocking playback start
- Added `resampleArray()` to `ui-logic.ts`, updated `drawWaveformToCanvas` to resample instead of regenerating synthetic data
- Deferred waveform loading to after playback starts
- Added waveform show/hide toggle button (waveform icon in transport bar)
- Moved progress-track/fill outside waveform-panel to always be visible at bottom of player
- Removed fixed 148px height from `.bottom` — auto-sizes to content (transport bar + waveform panel + progress bar)
- Changed `.waveform-panel` to fixed 108px height; when hidden, `.bottom` shrinks to just transport bar + progress bar
- **Fixed app freeze during waveform loading** — made `get_waveform_data` async with `tokio::task::spawn_blocking`. Previously it blocked a tokio worker thread, starving playback status polling and other IPC. Now runs on tokio's dedicated blocking thread pool (512 max threads)
- **Fixed stale waveform on track switch** — `waveRef.current` cleared immediately in waveform-loading effect before invoking backend
- **Replaced synthetic sine placeholder with flat dotted line** — while waveform is loading, canvas shows `Array(200).fill(0.02)` (tiny bars) instead of fake sine waves

---

## 10. Session Priority (Next Session)

### Immediate Next Steps
1. **Wire TrackList to real database** — Create `list_files` and `get_file` Tauri commands, update `App.tsx` to call them on mount and after scan
2. **Implement filename parser** (`analysis/parser.rs`) — Extract BPM, key, artist from filenames during scanning
3. **Wire search to SQLite** — Implement `search_files` command with proper indexing

### Medium Priority
4. Implement audio analysis fallback (Symphonia + stratum-dsp for BPM/key)
5. Add click-to-seek on waveform (Rodio Sink doesn't support seeking — would need source recreation at target position)
6. **Store duration during initial scan** — Probe duration with Symphonia while scanning so every file has accurate duration from the very first play

### Lower Priority (but needed)
7. Tag CRUD commands
8. Bundle detection
9. File watcher
10. Settings persistence
11. Clean install/uninstall configuration

---

## 11. Known Issues & Gotchas

- **Mock data not from DB:** TrackList populates with mock/mapped scan results, never reads from `list_files` command
- **Synthetic waveform:** `ui-logic.ts` generates fake waveform data using combined sine waves — not real audio samples
- **Hardcoded duration:** PlayerBar uses static `154` seconds for all tracks
- **`chrono_from_system_time` in filesystem.rs:** Custom ISO 8601 formatting is fragile (doesn't account for leap years, doesn't use chrono crate) — consider replacing with proper chrono or time crate
- **No `noUnusedLocals` exemption:** tsconfig has strict unused locals/params checks — currently some stubs trigger warnings (the existing code may not compile under tsc --noEmit cleanly)
- **Scan is synchronous:** `scan_directory` blocks the UI thread for large directories — needs tokio + progress channel
- **Duration unknown for MP3 files (first play only):** First play of a file uses file-size estimate (`file_size / 24000`) until waveform decode (~5s) returns the real duration. On subsequent plays, the stored duration from `audio_files.duration_secs` is used — progress is accurate from second one
- **No click-to-seek on waveform:** Rodio Sink doesn't support seeking. Would need to recreate the Sink from a source started at the target position
- **Waveform decoding is synchronous and slow (~5s for MP3):** `get_waveform_data` decodes entire files on a background thread (`tokio::task::spawn_blocking`). Doesn't block IPC or polling. Flat dotted line shown as placeholder while decoding
- **Tailwind not used in components:** All styling is via `index.css` classes, no Tailwind utility classes in JSX
- **Duplicate `analysis` and `db/models.rs` `ParsedMetadata`:** Both define similar metadata types — keep `db::models::ParsedMetadata` as the canonical type, use it from `analysis/parser.rs`
- **Edit the `analysis/parser.rs` test:** The placeholder test `test_parse_bpm` asserts `None` — update it once the parser is implemented

---

## 12. File Manifest (Key Files)

| File | Purpose |
|------|---------|
| `PROJECT-INFO.md` | **This file** — project guide, session log, roadmap |
| `NOTES.md` | Previous session notes (legacy) |
| `README.md` | Short project description |
| `package.json` | Node dependencies + scripts |
| `vite.config.ts` | Vite config (port 1420, Tauri HMR) |
| `tsconfig.json` | TypeScript config (strict, ES2021) |
| `tailwind.config.js` | Tailwind content paths |
| `run.bat` | Quick dev start script |
| `src-tauri/Cargo.toml` | Rust dependencies + build config |
| `src-tauri/tauri.conf.json` | Tauri app config (window, bundle, CSP) |
| `src-tauri/capabilities/default.json` | Tauri permissions (core + dialog) |
| `src-tauri/build.rs` | Tauri build script |
| `src-tauri/src/main.rs` | Rust entry point |
| `src-tauri/src/lib.rs` | Tauri setup, state, command registration |
| `src-tauri/src/config.rs` | Data directory paths |
| `src-tauri/src/error.rs` | AppError enum |
| `src-tauri/src/db/migrations.rs` | SQL schema |
| `src-tauri/src/db/models.rs` | Rust structs for DB entities |
| `src-tauri/src/db/pool.rs` | r2d2 connection pool |
| `src-tauri/src/scanner/filesystem.rs` | Directory scanner |
| `src/App.tsx` | Root React component |
| `src/index.css` | All styles (521 lines) |
| `src/components/Titlebar.tsx` | Scan button + app title |
| `src/components/Toolbar.tsx` | Filters + search |
| `src/components/TrackList.tsx` | Track list with stems |
| `src/components/PlayerBar.tsx` | Player with waveform |
| `src/lib/ui-logic.ts` | Utilities + stubs |
