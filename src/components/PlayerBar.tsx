import { useRef, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { formatTime, drawWaveformToCanvas } from '../lib/ui-logic';
import type { Track } from './TrackList';

interface PlaybackStatus {
  playing: boolean;
  paused: boolean;
  stopped: boolean;
  position: number;
  duration: number;
  file_path: string;
  volume: number;
}

interface WaveformData {
  peaks: number[];
  duration: number;
}

interface PlayerBarProps {
  selectedTrack: Track | null;
  onNext?: () => void;
  onPrev?: () => void;
}

export default function PlayerBar({ selectedTrack, onNext, onPrev }: PlayerBarProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const playheadRef = useRef<HTMLDivElement>(null);
  const progressFillRef = useRef<HTMLDivElement>(null);
  const waveRef = useRef<number[] | null>(null);
  const flatWaveRef = useRef<number[]>(Array(200).fill(0.02));
  const lastWavePathRef = useRef('');
  const pollRef = useRef<number>(0);
  const prevRestartedRef = useRef(false);
  const [showWaveform, setShowWaveform] = useState(false);
  const showWaveformRef = useRef(showWaveform);
  useEffect(() => { showWaveformRef.current = showWaveform; }, [showWaveform]);
  const [status, setStatus] = useState<PlaybackStatus>({
    playing: false,
    paused: false,
    stopped: true,
    position: 0,
    duration: 0,
    file_path: '',
    volume: 0.8,
  });

  useEffect(() => {
    pollRef.current = window.setInterval(async () => {
      try {
        const s = await invoke<PlaybackStatus>('get_playback_status');
        setStatus(s);
        updateVisuals(s);
      } catch (e) {
        console.error('Poll playback status failed:', e);
      }
    }, 200);

    return () => {
      if (pollRef.current) {
        clearInterval(pollRef.current);
      }
    };
  }, []);

  function updateVisuals(s: PlaybackStatus) {
    const progress = s.duration > 0 ? Math.min(s.position / s.duration, 1) : 0;

    if (progressFillRef.current) {
      progressFillRef.current.style.width = `${progress * 100}%`;
    }

    if (!showWaveformRef.current) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const parent = canvas.parentElement;
    if (!parent) return;

    const wave = waveRef.current ?? flatWaveRef.current;
    drawWaveformToCanvas(canvas, wave, progress);

    if (playheadRef.current) {
      const canvasRect = canvas.getBoundingClientRect();
      const parentRect = parent.getBoundingClientRect();
      const offsetLeft = canvasRect.left - parentRect.left;
      const canvasWidth = canvasRect.width;
      playheadRef.current.style.left = `${offsetLeft + canvasWidth * progress}px`;
    }
  }

  // Merged effect for updates that come from waveform decode (duration changes)
  useEffect(() => {
    updateVisuals(status);
  }, [status.position, status.duration, showWaveform]);

  useEffect(() => {
    waveRef.current = null;
    lastWavePathRef.current = '';
  }, [selectedTrack]);

  useEffect(() => {
    if (!status.playing || !selectedTrack?.path) return;
    if (waveRef.current && lastWavePathRef.current === selectedTrack.path) return;
    waveRef.current = null;
    lastWavePathRef.current = selectedTrack.path;
    const requestedPath = selectedTrack.path;
    invoke<WaveformData>('get_waveform_data', { path: requestedPath, bars: 500 })
      .then((data) => {
        if (lastWavePathRef.current !== requestedPath) return;
        waveRef.current = data.peaks;
        if (data.duration > 0) {
          setStatus((prev) => ({ ...prev, duration: data.duration }));
          invoke('set_duration', { duration: data.duration });
          invoke('store_track_duration', { path: requestedPath, duration: data.duration });
        }
      })
      .catch((e) => console.error('Waveform fetch failed:', e));
  }, [status.playing, selectedTrack]);

  useEffect(() => {
    const onResize = () => {
      waveRef.current = null;
    };
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  const handleTogglePlay = async () => {
    if (!selectedTrack) return;
    prevRestartedRef.current = false;
    try {
      const s = await invoke<PlaybackStatus>('toggle_playback', { path: selectedTrack.path });
      setStatus(s);
    } catch (e) {
      console.error('Playback toggle failed:', e);
    }
  };

  const handlePrev = async () => {
    if (!selectedTrack) return;

    if (prevRestartedRef.current) {
      prevRestartedRef.current = false;
      onPrev?.();
      return;
    }

    if (status.playing || status.position > 0.5) {
      prevRestartedRef.current = true;
      try {
        const s = await invoke<PlaybackStatus>('play_audio', { path: selectedTrack.path });
        setStatus(s);
      } catch (e) {
        console.error('Restart failed:', e);
      }
    } else {
      onPrev?.();
    }
  };

  const handleNext = () => {
    prevRestartedRef.current = false;
    onNext?.();
  };

  const handleVolume = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const vol = parseFloat(e.target.value) / 100;
    setStatus((prev) => ({ ...prev, volume: vol }));
    try {
      await invoke('set_volume', { volume: vol });
    } catch (err) {
      console.error('Volume change failed:', err);
    }
  };

  const handleSeek = async (e: React.MouseEvent<HTMLDivElement | HTMLCanvasElement>) => {
    if (status.duration <= 0) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const fraction = Math.max(0, Math.min(x / rect.width, 1));
    const targetPos = fraction * status.duration;
    try {
      const s = await invoke<PlaybackStatus>('seek_audio', { position: targetPos });
      setStatus(s);
      updateVisuals(s);
    } catch (err) {
      console.error('Seek failed:', err);
    }
  };

  const isPlaying = status.playing;
  const time = formatTime(status.position);
  const total = formatTime(status.duration);
  const volPct = Math.round(status.volume * 100);
  const metaText = selectedTrack
    ? `${selectedTrack.bpm ?? '—'} BPM · ${selectedTrack.key} · ${selectedTrack.artists}`
    : '';

  return (
    <div className="bottom">
      <div className="transport-bar">
        <div className="track-info">
          <div className="track-info-name">{selectedTrack?.name ?? 'No track selected'}</div>
          <div className="track-info-meta">{metaText}</div>
        </div>
        <div className="sep" />
        <div className="t-controls">
          <button className="t-btn" onClick={handlePrev} title="Previous / Restart">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <rect x="1.5" y="2" width="2" height="10" rx="0.5" />
              <path d="M12.5 2L5.5 7L12.5 12V2Z" />
            </svg>
          </button>
          <button className="t-btn play-pause" onClick={handleTogglePlay} title={isPlaying ? 'Pause' : 'Play'}>
            {isPlaying ? (
              <svg width="13" height="13" viewBox="0 0 13 13" fill="currentColor">
                <rect x="2" y="1.5" width="3.5" height="10" rx="1" />
                <rect x="7.5" y="1.5" width="3.5" height="10" rx="1" />
              </svg>
            ) : (
              <svg width="13" height="13" viewBox="0 0 13 13" fill="currentColor">
                <path d="M2.5 1.5L11.5 6.5L2.5 11.5V1.5Z" />
              </svg>
            )}
          </button>
          <button className="t-btn" onClick={handleNext} title="Next track">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <rect x="10.5" y="2" width="2" height="10" rx="0.5" />
              <path d="M1.5 2L8.5 7L1.5 12V2Z" />
            </svg>
          </button>
          <button className="t-btn" onClick={() => setShowWaveform(!showWaveform)} title={showWaveform ? 'Hide waveform' : 'Show waveform'}>
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <path d="M2 1v12" stroke="currentColor" strokeWidth="1.5" />
              <path d="M5 4v6" stroke="currentColor" strokeWidth="1.5" />
              <path d="M8 2v10" stroke="currentColor" strokeWidth="1.5" />
              <path d="M11 5v4" stroke="currentColor" strokeWidth="1.5" />
            </svg>
          </button>
        </div>
        <div className="sep" />
        <div className="time-counter">{time}{status.duration > 0 ? ` / ${total}` : ''}</div>
        <div className="vol-row">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M3 5H1v4h2l4 3V2L3 5Z" fill="currentColor" />
            <path d="M10 4.5c.8.6 1.3 1.5 1.3 2.5s-.5 1.9-1.3 2.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
            <path d="M12 2.5c1.2 1.1 1.8 2.7 1.8 4.5s-.6 3.4-1.8 4.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
          <input
            type="range"
            className="vol-slider"
            min="0"
            max="100"
            value={volPct}
            onChange={handleVolume}
          />
          <span style={{ fontSize: 11, color: 'var(--text2)', minWidth: 32, textAlign: 'right' }}>
            {volPct}%
          </span>
        </div>
      </div>
      <div className={`waveform-panel${showWaveform ? '' : ' hidden'}`}>
        <canvas ref={canvasRef} id="waveform-canvas" onClick={handleSeek} />
        <div className="playhead-line" ref={playheadRef} />
      </div>
      <div className="progress-track" onClick={handleSeek}>
        <div className="progress-fill" ref={progressFillRef} />
      </div>
    </div>
  );
}
