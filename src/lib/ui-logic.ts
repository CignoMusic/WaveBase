// ─── Formatting ───

export function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${String(s).padStart(2, '0')}`;
}

// ─── Waveform ───
// TODO: Replace synthetic generation with real audio sample data from Symphonia decoding

export function resampleArray(data: number[], targetLen: number): number[] {
  if (data.length === 0 || targetLen === 0) return Array(targetLen).fill(0);
  const result = new Array(targetLen);
  for (let i = 0; i < targetLen; i++) {
    const pos = (i / targetLen) * data.length;
    const idx = Math.min(Math.floor(pos), data.length - 1);
    result[i] = data[idx];
  }
  return result;
}

export function generateWaveformData(bars: number): number[] {
  const d: number[] = [];
  for (let i = 0; i < bars; i++) {
    let h = Math.abs(Math.sin(i * 0.21)) * 0.44
      + Math.abs(Math.sin(i * 0.074 + 0.9)) * 0.26
      + Math.abs(Math.sin(i * 0.52 + 2.0)) * 0.18
      + Math.abs(Math.sin(i * 1.38 + 0.4)) * 0.12;
    d.push(Math.max(0.05, Math.min(1, h)));
  }
  for (let i = 1; i < d.length - 1; i++) {
    d[i] = d[i] * 0.68 + (d[i - 1] + d[i + 1]) * 0.16;
  }
  return d;
}

export function drawWaveformToCanvas(
  canvas: HTMLCanvasElement,
  waveformData: number[],
  progress: number,
): void {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const parent = canvas.parentElement;
  if (!parent) return;

  const rect = parent.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  const W = rect.width - 24;
  const H = rect.height - 12;

  canvas.width = W * dpr;
  canvas.height = H * dpr;
  canvas.style.width = `${W}px`;
  canvas.style.height = `${H}px`;
  ctx.scale(dpr, dpr);

  const BAR = 2, GAP = 1, STEP = BAR + GAP;
  const bars = Math.floor(W / STEP);
  const data = waveformData.length === bars ? waveformData : resampleArray(waveformData, bars);

  const px = W * progress;
  ctx.clearRect(0, 0, W, H);

  for (let i = 0; i < Math.min(bars, data.length); i++) {
    const x = i * STEP;
    const bh = data[i] * H * 0.80;
    const y = (H - bh) / 2;
    const played = x < px;
    const nearHead = Math.abs(x - px) < 10;
    if (played) {
      ctx.fillStyle = nearHead ? 'rgba(229,229,231,0.90)' : 'rgba(229,229,231,0.55)';
    } else {
      ctx.fillStyle = nearHead ? 'rgba(229,229,231,0.22)' : 'rgba(229,229,231,0.13)';
    }
    ctx.fillRect(x, y, BAR, bh);
  }
}

// ─── Playback (stub) ───
// TODO: Wire to Tauri play_audio / stop_audio / pause_audio commands + Rodio thread

export interface PlaybackState {
  playing: boolean;
  progress: number;
  duration: number;
  volume: number;
}

export function createPlaybackState(): PlaybackState {
  return { playing: false, progress: 0, duration: 0, volume: 0.8 };
}

// ─── Search / Filter (stub) ───
// TODO: Wire to search_files Tauri command with debounced SQLite queries

export interface SearchState {
  query: string;
  activeFilter: string;
  results: unknown[];
  isSearching: boolean;
}

export function createSearchState(): SearchState {
  return { query: '', activeFilter: 'All', results: [], isSearching: false };
}

// ─── File Scanning (stub) ───
// TODO: Wire to scan_directory / scan_status Tauri commands.
//       Use Tauri dialog API or HTML input.webkitdirectory to pick folder.

export interface ScanState {
  isScanning: boolean;
  progress: number;
  lastScan: string | null;
}

export function createScanState(): ScanState {
  return { isScanning: false, progress: 0, lastScan: null };
}

// ─── Library (stub) ───
// TODO: Wire to list_files / get_file / search_files Tauri commands

export interface LibraryFile {
  id: string;
  name: string;
  path: string;
  bpm: number | null;
  key: string | null;
  artist: string | null;
  duration: number;
  extension: string;
  folder: string;
}

export function createLibraryState(): { files: LibraryFile[]; total: number } {
  return { files: [], total: 0 };
}

// ─── Tags ───

export interface TagInfo {
  id: number;
  name: string;
  color: string | null;
  isPreset: boolean;
  isPinned: boolean;
  isMetadata: boolean;
}

export interface TagProgress {
  total: number;
  processed: number;
  status: string;
}

export interface ScanRoot {
  id: number;
  path: string;
}
