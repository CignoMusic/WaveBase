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
│   ├── scan.rs          # scan_directory, get_tag_progress — async background tagging pipeline (4 workers, skip-if-analyzed)
│   ├── library.rs       # list_files, get_file — search_files (stub)
│   ├── playback.rs      # play_audio, toggle_playback, pause_audio, resume_audio, stop_audio, get_playback_status, set_volume, set_duration, store_track_duration, get_waveform_data, seek_audio
│   └── tags.rs          # add_tag, remove_tag, list_file_tags, get_all_tags, get_pinned_tags, toggle_tag_pin, create_tag, delete_tag, filter_files_by_tag_names, get_tag_file_count
│
├── db/                  # Database layer
│   ├── mod.rs           # Module declarations
│   ├── models.rs        # AudioFile, Tag, FileTag, ScanRoot, SearchQuery, AudioFileWithTags, TagProgress, ParsedMetadata
│   ├── migrations.rs    # Schema: audio_files, tags (is_preset, is_pinned, is_metadata), file_tags, scan_roots + indexes
│   └── pool.rs          # r2d2 SqliteConnectionManager pool (max 4 connections)
│
├── scanner/             # File system scanning
│   ├── mod.rs           # Module declarations
│   ├── filesystem.rs    # walkdir-based recursive scan, audio extension filter
│   └── watcher.rs       # notify-based file watcher (stub — not yet used)
│
├── playback/            # Audio playback
│   ├── mod.rs           # Module declarations
│   ├── player.rs        # AudioPlayer with Rodio — play, pause, resume, stop, toggle, volume, status, seek (PCM pre-decode)
│   └── waveform.rs      # compute_waveform_peaks — Symphonia peak extraction for frontend canvas
│
└── analysis/            # Audio metadata analysis
    ├── mod.rs           # Module declarations
    ├── parser.rs        # Smart filename parsing — BPM, key, track type, @artist extraction
    ├── decoder.rs       # Symphonia audio decoding to mono f32 PCM
    └── dsp.rs           # stratum-dsp BPM + key detection with confidence threshold
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
│   ├── Titlebar.tsx     # Top bar: app name + gear (settings) + "Scan Directory" button (uses @tauri-apps/plugin-dialog)
│   ├── Toolbar.tsx      # Pinned tag pills + Filter dropdown (all tags) + search input + progress bar
│   ├── TrackList.tsx    # Scrollable track list with collapsible stems, tags column, analyzed(~) indicator, inline tag add/remove
│   ├── SettingsPanel.tsx# Tag manager modal: add/delete tags, pin/unpin toggle, file counts
│   └── PlayerBar.tsx    # Bottom player: transport controls, waveform canvas, progress, volume
│
└── lib/
    └── ui-logic.ts      # Utilities: formatTime, waveform generation/rendering, TagInfo/TagProgress types
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

## 5. Current State (Session 10)

### ✅ Working / Stable
- App launches, shows full UI
- "Scan Directory" picks a folder, scans for audio files, returns results
- Files inserted into SQLite `audio_files` table
- UI renders: Titlebar, Toolbar with filters, TrackList, PlayerBar with waveform canvas
- Selection state, stem expansion work
- Database connection pool, migrations, schema all set up
- Custom dark theme CSS complete (no Tailwind utility classes used yet)
- Clean app data dir management (%APPDATA%/wavebase, ~/Library/Application Support/wavebase)
- **Real audio playback via Rodio** — play, pause, resume, stop, volume control all working
- **Playback position & duration tracking** — position counter, playhead, and progress fill all update in real time
- **Next/prev track navigation** — prev restarts track first click, then goes to previous; next wraps around
- **Smart prev button** — first click restarts current track, second click goes to previous
- **Real waveform from audio data** — waveform peaks extracted via Symphonia on the Rust backend, replaces synthetic sine waves
- **DB-stored track duration** — real duration saved to `audio_files.duration_secs` when waveform decode completes; used on subsequent plays for instant accurate progress
- **Click-to-seek** — click on progress bar or waveform canvas to seek to any position; uses pre-decoded PCM buffer for instant O(1) seeking
- **Keyboard shortcuts** — Space/MediaPlayPause toggles play/pause, MediaNextTrack/MediaPreviousTrack and Ctrl+Arrow for track navigation; suppressed when typing in input fields
- **Filename parser** — `analysis/parser.rs` extracts BPM, key, track type, @artist(s) from filenames; conservative matching, tokens in any order
- **Audio analysis fallback** — `analysis/decoder.rs` (Symphonia PCM decode) + `analysis/dsp.rs` (stratum-dsp BPM/key) fills gaps when filename doesn't provide metadata
- **Tag CRUD** — `commands/tags.rs`: add, remove, list, create, delete, AND-filter, pin/unpin, get_pinned_tags, get_tag_file_count
- **Async background tagging** — scan returns instantly, then 4 parallel background threads parse filenames + analyze audio + auto-create metadata tags; progress bar polls every 300ms
- **Pinned filter pills** — Toolbar shows only PINNED tags as clickable pills; Filter dropdown shows ALL tags with checkboxes; AND-matching
- **Tag manager settings** — `SettingsPanel.tsx`: add/delete tags, pin/unpin toggle, file counts, preset badge
- **Settings gear button** — `Titlebar.tsx`: gear icon opens settings modal
- **Analyzed indicator (~)** — BPM/key from audio analysis show ~ icon (tooltip: "From audio analysis")
- **TrackList persistence** — `list_files` reads from DB; tracks persist across restarts
- **Metadata tags hidden from tags column** — BPM/Key/Duration auto-tags (is_metadata=1) not shown in tags column; BPM/Key have their own columns
- **Auto-detected tags** — BPM ("140 BPM"), Key ("D Minor"), Duration ("0:32"), Artist ("@username") auto-created and marked as metadata; track type tags (Loop, Beat, Stem, OneShot) are presets, pinned by default
- **Background tagging progress** — Toolbar shows progress bar "Tagging: 3/5" → "✓ Tagged" when complete; files re-fetched from DB to show analyzed BPM/key values
- **Parallel background processing** — 4 worker threads for faster tagging with `AtomicUsize` progress tracking
- **DB migrations** — `PRAGMA table_info` checks + `ALTER TABLE ADD COLUMN` for upgrading existing DBs; seeds preset tags + pins

### ⚠️ Partial / Needs Wiring
| Component | Issue | Details |
|-----------|-------|---------|
| `Toolbar.tsx` search | Frontend-only | Search input renders but `search_files` backend is still a stub — no debounced SQL query yet |
| `ui-logic.ts` play stubs | Legacy | Playback/search/scan state factories exist but aren't used (real data flows through Tauri invoke directly) |

### ⬜ Not Yet Implemented (Stubs)
| Module | Files | What's Missing |
|--------|-------|----------------|
| **File watcher** | `scanner/watcher.rs` | Real-time filesystem monitoring via notify |
| **Bundle detection** | Not started | Group related files by folder |
| **Settings persistence** | `config.rs` has path | No TOML read/write yet |
| **Search** | `commands/library.rs:search_files` | Proper SQLite full-text search across filename, tags, BPM, key, artist |
| **Customizable columns** | Not started | Add/remove/reorder tag columns (TrackList grid is hardcoded) |

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

### Feature 2: Smart Filename Parsing & Auto-Tagging ✅ (Done)
- [x] Parse filename for BPM (pattern: `123bpm`, `123 BPM`, case-insensitive)
- [x] Parse filename for key (pattern: `D Minor`, `Dmin`, `Dm`, `D#maj`, normalize to full notation)
- [x] Parse filename for artist (pattern: `@username`)
- [x] Extract track name (remaining text after removing structured tokens)
- [x] Treat underscores/dashes as spaces
- [x] Case-insensitive matching for BPM and key
- [x] Tokens can appear in any position in filename
- [x] Store parsed values in DB fields (bpm, key, artist, track_name)
- [x] Parsed values automatically become editable tags

### Feature 3: Audio Analysis Fallback ✅ (Done)
- [x] Decode audio with Symphonia (WAV, MP3, AIFF, FLAC, OGG, M4A)
- [x] BPM detection via stratum-dsp
- [x] Key detection via stratum-dsp
- [x] Only analyze fields not already parsed from filename
- [x] Skip analysis if filename already provided the value (keep scanning fast)
- [x] Handle analysis errors gracefully (leave field empty for manual tagging)

### Feature 4: Audio Preview ✅ (Position tracking fixed)
- [x] Play audio via Rodio triggered by Tauri commands
- [x] Play/pause/resume/stop controls wired to real playback
- [x] Volume control (Rodio Sink::set_volume)
- [x] Position polling from backend every 200ms
- [x] Backend tracks position accounting for pauses
- [x] Auto-detect when playback finishes (Sink::empty)
- [x] Real waveform from audio data (Symphonia peak extraction, canvas rendering)
- [x] Per-file duration from file headers (lofty, instant), decoded audio (Symphonia), or DB cache
- [x] Click-to-seek on waveform/progress bar (PCM pre-decode for instant seek)
- [x] Global keyboard shortcuts (Space/MediaPlayPause, MediaNext/Prev, Ctrl+Arrow)

### Feature 5: Advanced Manual Tagging ✅ (Done)
- [x] Add/remove/list tags per file
- [x] Tag management (add, delete tags in settings panel)
- [x] Multi-tag AND-filtering
- [x] Auto-detected values stored as editable tags
- [x] Pinned/unpinned tag pills (toolbar only shows pinned)
- [x] Metadata tags (BPM/Key/Artist) hidden from tags column
- [x] Settings gear button in titlebar
- [x] Pin/unpin toggle in tag manager
- [x] Background parallel tagging (4 workers) with progress bar
- [x] Analyzed indicator (~) for analysis-originated values
- [x] Multiple @artist detection and tagging
- [x] DB migration logic (PRAGMA + ALTER TABLE) for existing databases
- [ ] Tag colors (schema supports it, UI deferred)
- [ ] Tag rename/merge (settings panel deferred)
- [ ] Customizable columns (add/remove/reorder tag columns — deferred to future session)

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
- Grid layout for track rows: `32px 28px 1fr 72px 96px 1fr minmax(100px, 1.2fr)` (drag-handle, dot, name, bpm, key, artists, tags)

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

### Session 10 — Pinning, Metadata Tags, Parallel Processing & Frontend Polish

**Backend — DB Schema & Migrations (`db/migrations.rs`, `db/models.rs`):**
- `tags` table: added `is_pinned INTEGER NOT NULL DEFAULT 0`, `is_metadata INTEGER NOT NULL DEFAULT 0`
- `ParsedMetadata`: `artists` changed from `Option<String>` to `Vec<String>` for multiple @artist support
- Migration uses `PRAGMA table_info` + `ALTER TABLE ADD COLUMN` for existing DB upgrades
- Seeds preset tags (Loop, Beat, Stem, OneShot) with `is_pinned=1` via `INSERT OR IGNORE`

**Backend — Tags Commands (`commands/tags.rs`):**
- Added `get_pinned_tags` — returns only tags where `is_pinned=1`
- Added `toggle_tag_pin` — flips `is_pinned`, returns updated `Tag`
- `list_file_tags` accepts optional `exclude_metadata` parameter
- `create_tag` sets `is_preset=0, is_pinned=0` for user-created tags
- Auto-created BPM/Key/Duration/Artist tags now set `is_metadata=1`
- `Tag` struct now includes `is_preset`, `is_pinned`, `is_metadata` fields
- `TAG_COLS` constant keeps all SELECT/INSERT column lists consistent

**Backend — Background Tagging (`commands/scan.rs`):**
- **Parallel processing:** replaced serial loop with 4 concurrent worker threads using `Arc<Vec<Mutex>>` for DB pools
- **Skip-if-analyzed:** files with both bpm and key already set skip audio analysis entirely (speedup for re-scans)
- **Progress tracking:** swapped `Arc<Mutex<TagProgress>>` for `Arc<(AtomicUsize, AtomicUsize)>` — lock-free progress reads
- Auto-tags for BPM ("140 BPM", `is_metadata=1`), Key ("D Minor", `is_metadata=1`), Duration ("0:32", `is_metadata=1`), Artist ("@username", `is_metadata=1`)
- Track type tags (Loop, Beat, Stem, OneShot) remain `is_metadata=0`, `is_preset=1`

**Backend — Filename Parser (`analysis/parser.rs`):**
- Multiple @artist support: `artists: Vec<String>` collects all `@mention` tokens
- Added `test_parse_multiple_artists` test (11 total parser tests, all pass)

**Frontend — App.tsx:**
- Computes `pinnedTags = allTags.filter(t => t.isPinned)` for toolbar pills
- Added `loadAllFileTagsRef` to avoid stale closure in progress polling interval
- After background tagging completes: re-fetches file list from DB to show updated BPM/key values
- Passes `pinnedTags` + `allTags` separately to Toolbar
- Passes `onOpenSettings` to Titlebar for gear button

**Frontend — Toolbar.tsx:**
- Now accepts `pinnedTags` and `allTags` as separate props
- Pills show only `pinnedTags` (no slice/limit — all pinned tags shown)
- Filter dropdown still shows ALL `allTags` with checkboxes
- Removed the "+N" overflow button (no longer needed — pinned set is curated)

**Frontend — TrackList.tsx:**
- Tags column filters display tags with `displayTags(track)` → `getTags(track).filter(t => !t.isMetadata)`
- Metadata tags (BPM/Key/Duration/Artist) hidden from tags column
- Add-tag dropdown still uses `getTags(track)` (all tags) for "already have" check
- Import uses `TagInfo` from `ui-logic.ts`

**Frontend — Titlebar.tsx:**
- Added `onOpenSettings` prop
- Gear icon button before "Scan Directory": opens settings modal for tag management

**Frontend — SettingsPanel.tsx:**
- Added `handleTogglePin(tagId)` calling `toggle_tag_pin` command
- Each tag row shows "Pin" / "Pinned" toggle button
- Pinned state reflected in real time (blue "Pinned" text)
- Layout kept minimal: name, file count, pin toggle, delete/preset badge

**Frontend — index.css:**
- `.settings-pin-btn` — pin/unpin toggle styling with `.pinned` modifier (blue text)
- `.titlebar-actions` already flex-row, gear button sits left of scan button

### Session 9 — Tagging & Filtering System

**Backend — Filename Parser (`analysis/parser.rs`):**
- Implemented `parse_filename()` extracting BPM (`\d{2,3}bpm`, `bpm\d{2,3}`), key (`[A-G][#b]? (maj|min|m)`), track type (loop/beat/stem/oneshot), artist (@username)
- Tokens can appear anywhere in filename, case-insensitive, underscores/dashes treated as spaces
- Conservative matching — leaves fields empty on ambiguity (no guessing)

**Backend — Audio Analysis (`analysis/decoder.rs`, `analysis/dsp.rs`):**
- `analysis/decoder.rs`: Full Symphonia decode to mono `Vec<f32>` + sample rate (reuses pattern from `playback/waveform.rs`)
- `analysis/dsp.rs`: stratum-dsp `analyze_audio()` integration for BPM + key detection with `AnalysisConfig::default()`
- Only runs for fields not already parsed from filename (skip if filename provided the value)
- Errors handled gracefully — leaves field empty rather than crashing

**Backend — Tag CRUD (`commands/tags.rs`):**
- `add_tag` — find-or-create tag by name + link to file via `file_tags`
- `remove_tag` — unlink tag from file
- `list_file_tags` — all tags for a given file path
- `get_all_tags` — all tags in the system (for filter pills)
- `get_pinned_tags` / `toggle_tag_pin` — pin management for toolbar
- `create_tag` / `delete_tag` — explicit tag management (for settings panel)
- `filter_files_by_tag_names` — AND-match: file must have ALL specified tags
- `get_tag_file_count` — number of files tagged with a given tag

**Backend — Async Background Tagging (`commands/scan.rs`):**
- `scan_directory` stays fast: walkdir + INSERT files, spawns background thread, returns immediately
- Background thread runs per file: parse_filename → UPDATE DB + auto-create + link tags → if gaps → decode_audio → analyze → UPDATE with bpm_analyzed=1 → create tags
- Lofty probe_duration creates time-code tags (e.g., "0:32")
- `TagProgress` shared state (`Arc<Mutex<>>`) tracks total/processed/status
- `get_tag_progress` command for frontend polling
- State initialized in Tauri `setup()`

**Backend — DB Schema:**
- `audio_files` gained `bpm_analyzed INTEGER` and `key_analyzed INTEGER` (0=from filename, 1=from analysis)
- `tags` gained `is_preset INTEGER` (1=system tag, cannot be deleted)
- Preset tags seeded on first migration: Loop, Beat, Stem, OneShot

**Frontend — App.tsx:**
- State: `allTags`, `activeTagNames`, `tagProgress`, `showSettings`, `searchQuery`
- On mount: `get_all_tags` + `list_files` (persistent track list across app restarts)
- After scan: starts polling `get_tag_progress` every 300ms
- Filtering: AND-match local tracks against `activeTagNames`
- Handlers: onTagToggle, onAddTag, onRemoveTag, onOpenSettings

**Frontend — Toolbar.tsx:**
- Dynamic filter pills rendered from `allTags` (no hardcoded Beats/Loops/Stems)
- Active filters highlighted with simple active class
- "Filter" button → dropdown with checkboxes per tag + "Manage Tags…" link to settings
- Compact progress bar shown when `tagProgress.status === "scanning"`
- Search input → `onSearchChange` callback

**Frontend — TrackList.tsx:**
- Added 7th "Tags" column
- Row hover shows "+" button → opens mini tag picker dropdown
- Tag names have hover "×" to remove from file
- Analyzed indicator: small `~` after BPM/key values when `bpm_analyzed=1`/`key_analyzed=1`

**Frontend — SettingsPanel.tsx:**
- Modal overlay for tag management
- Lists all tags with name + file count + delete button
- Preset tags shown with locked badge, delete disabled
- "Add Tag" form (name input)
- Delete shows confirmation with file count

**Frontend — index.css:**
- `.tag-list`, `.tag-add-btn`, `.tag-dropdown`, `.analysis-hint` — tag column styles
- `.filter-dropdown`, `.filter-option`, `.filter-dropdown-footer` — multi-select filter dropdown
- `.tag-progress`, `.tag-progress-bar`, `.tag-progress-fill`, `.tag-progress-done` — progress bar
- `.settings-overlay`, `.settings-panel`, `.settings-row`, `.settings-row-actions` — settings modal
- Updated `.col-header` / `.row` grid to 7 columns (added tags column)

### Session 8 — Global Keyboard Shortcuts
- **Added global keydown handler** in `App.tsx` — listens for `Space`, `MediaPlayPause`, `MediaNextTrack`, `MediaPreviousTrack`, and `Ctrl+ArrowLeft`/`Ctrl+ArrowRight`
- **Space / MediaPlayPause** → toggles play/pause on the currently selected track via `toggle_playback`
- **MediaNextTrack / Ctrl+ArrowRight** → skips to next track (wraps around)
- **MediaPreviousTrack / Ctrl+ArrowLeft** → skips to previous track (wraps around)
- **Input-aware** — shortcuts are suppressed when focus is in an `<input>` or `<textarea>` (e.g., search box)
- **Extracted `handleTogglePlay`** into `App.tsx` as a reusable callback, shared by the keyboard handler

### Session 7 — Click-to-Seek on Progress Bar & Waveform
- **Added `Command::Seek` to audio thread** — recreates the Rodio sink at an arbitrary position (rewritten mid-session for performance)
- **Added `seek_offset` tracking** — `flush_state()` now accepts `seek_offset: f64` so position = `seek_offset + elapsed_since_seek - total_paused`. Resets timing variables correctly on seek
- **Added `AudioPlayer::seek()` method** and `seek_audio` Tauri command — returns the updated `PlaybackStatus` after seeking
- **Frontend click handlers** on both `.progress-track` (always visible) and `#waveform-canvas` — computes click fraction × `status.duration`, invokes `seek_audio`, updates visual state immediately via both `setStatus(s)` and `updateVisuals(s)`
- **Edge cases handled:** seek while paused (stays paused at new position), seek to position beyond duration (clamped), seek when no track loaded (no-op)
- **First iteration (slow):** Used Rodio's `source.skip_duration(position)` — decodes every sample from start to seek position before playback can resume (~2s delay for MP3)
- **Second iteration (instant):** Replaced with PCM pre-decode approach. After each Play/Toggle, a background thread decodes the entire file to raw PCM (Vec<f32>) via Symphonia. On seek, a `SamplesBuffer` is created from the PCM data starting at the seek position — O(1) seek, no decoding at seek time. The background decode runs in parallel with Rodio playback, so by the time the user seeks, the buffer is usually ready. Falls back to the old slow path if the buffer isn't ready yet (edge case for immediate seeks on newly played files)

### Session 6 — Accurate Duration from File Headers (No Guessing)
- **Replaced file-size estimate with real header-based duration probe** — Added `lofty` crate to Cargo.toml, created `probe_duration()` in `player.rs` that reads actual audio file metadata headers (instant, no decode)
- **Removed `estimate_duration()`** — the 192 kbps file-size guess is gone. All formats now get accurate duration from their headers:
  - WAV: RIFF header → data_size / byte_rate
  - FLAC: STREAMINFO → total_samples / sample_rate
  - MP3: Xing/Info VBR header → frame_count × samples_per_frame / sample_rate
  - AIFF: COMM chunk → num_sample_frames / sample_rate
  - OGG/M4A: container-level metadata
- **Kept DB-stored duration as cache layer** — `store_track_duration` and the DB lookup in `play_audio`/`toggle_playback` remain as a fallback/optimization if `lofty` can't read a format. Waveform decode (Symphonia) still stores the most authoritative duration in the DB
- **Result:** Every play of every file has accurate progress from second one. No guessing, no jumping, no DB dependency for correctness
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
1. **Customizable columns** — Add/remove/reorder tag columns in TrackList grid (currently hardcoded 7 columns). User can choose which columns to display and in what order. Settings panel option to configure columns.

### Medium Priority
2. **Wire search** — Implement `search_files` backend (SQLite LIKE query across filename/tags/BPM/key) + debounced frontend wiring
3. **Bundle detection** — Group related files by folder, collapsible UI
4. **File watcher** — Real-time filesystem monitoring via `notify`

### Lower Priority (but needed)
5. Settings persistence — TOML read/write via `config.rs`
6. Tag colors (already in schema, needs UI in settings)
7. Tag rename/merge (settings panel)
8. Clean install/uninstall configuration

---

## 11. Known Issues & Gotchas

- **Audio analysis is blocking (tokio::task::spawn_blocking):** `analysis/decoder.rs` and `analysis/dsp.rs` run on tokio's blocking thread pool. 4 parallel background worker threads mitigate this for large libraries, but 500+ files could still take minutes. Acceptable for v1.
- **stratum-dsp analysis may be inaccurate for non-musical content:** BPM/key detection works best on structured music. Sound effects, dialogue, or ambient recordings may produce unreliable results. The ~ indicator warns users.
- **`chrono_from_system_time` in filesystem.rs:** Custom ISO 8601 formatting is fragile (doesn't account for leap years, doesn't use chrono crate) — replace with proper chrono or time crate.
- **No `noUnusedLocals` exemption:** tsconfig has strict unused locals/params checks — currently some stubs trigger warnings.
- **Holding Ctrl+Arrow fires rapid track switching:** Keyboard handler has no debounce/throttle. Holding Ctrl+Arrow rapidly cycles through tracks, each firing play_audio + PCM decode. Acceptable for normal use.
- **Tailwind not used in components:** All styling is via `index.css` classes, no Tailwind utility classes in JSX.
- **Search not wired:** `search_files` command is still a stub — search input in Toolbar is frontend-only.
- **Tag rename/merge not implemented:** Settings panel only supports add/delete, not rename or merge tags.
- **Customizable columns deferred:** TrackList grid is hardcoded to 7 columns. Add/remove/reorder of tag columns is a future feature.
- **Stale `tracks` closure in interval callback (App.tsx:119):** The tag progress polling interval captures `tracks` via `loadAllFileTagsRef.current` to avoid stale closure issues; but if a scan completes while no interval is running, tracks could be stale. Acceptable for v1.
- **File re-fetch after tagging may not preserve UI state:** After background tagging completes, the entire track list is re-fetched from DB, which loses the scroll position and expansion state. Noted for future improvement.

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
| `src-tauri/src/db/migrations.rs` | SQL schema + migration logic (PRAGMA + ALTER TABLE for upgrades) |
| `src-tauri/src/db/models.rs` | Rust structs for DB entities (AudioFile, Tag, FileTag, TagProgress, ParsedMetadata) |
| `src-tauri/src/db/pool.rs` | r2d2 connection pool (max 4) |
| `src-tauri/src/scanner/filesystem.rs` | Directory scanner (walkdir) |
| `src-tauri/src/scanner/watcher.rs` | File watcher (stub, notify-based) |
| `src-tauri/src/analysis/parser.rs` | Filename BPM/key/type/artist parser (11 tests) |
| `src-tauri/src/analysis/decoder.rs` | Symphonia PCM decode to mono f32 |
| `src-tauri/src/analysis/dsp.rs` | stratum-dsp BPM + key detection |
| `src-tauri/src/commands/tags.rs` | Full tag CRUD + pin/filter/count commands |
| `src-tauri/src/commands/scan.rs` | Async background tagging (4 workers, progress) |
| `src/App.tsx` | Root React component — orchestrates scanning, filtering, tagging, selection |
| `src/index.css` | All styles (~860 lines) |
| `src/components/Titlebar.tsx` | App title + gear (settings) + scan button |
| `src/components/Toolbar.tsx` | Pinned pills + filter dropdown + progress bar + search |
| `src/components/TrackList.tsx` | 7-column track list with tags, analyzed indicator, inline tag add/remove |
| `src/components/SettingsPanel.tsx` | Tag management modal (add/delete, pin/unpin, file counts) |
| `src/components/PlayerBar.tsx` | Transport controls + waveform canvas + progress |
