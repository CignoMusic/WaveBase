import { useRef, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { formatTime, generateWaveformData, drawWaveformToCanvas } from '../lib/ui-logic';
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
  const pollRef = useRef<number>(0);
  const prevRestartedRef = useRef(false);
  const maxPositionRef = useRef(0);
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

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const wave = waveRef.current ?? generateWaveformData(200);
    waveRef.current = wave;
    if (status.duration > 0) {
      maxPositionRef.current = 0;
    } else if (status.position > maxPositionRef.current) {
      maxPositionRef.current = status.position;
    }
    const denom = status.duration > 0 ? status.duration : Math.max(maxPositionRef.current, status.position + 0.1);
    const progress = Math.min(status.position / denom, 1);
    drawWaveformToCanvas(canvas, wave, progress);

    const parent = canvas.parentElement;
    if (playheadRef.current && parent) {
      const w = parent.getBoundingClientRect().width - 24;
      playheadRef.current.style.left = `${12 + w * progress}px`;
    }
    if (progressFillRef.current) {
      progressFillRef.current.style.width = `${progress * 100}%`;
    }
  }, [status.position, status.duration]);

  useEffect(() => {
    waveRef.current = null;
    maxPositionRef.current = 0;
    if (selectedTrack?.path) {
      invoke<number[]>('get_waveform_data', { path: selectedTrack.path, bars: 500 })
        .then((data) => { waveRef.current = data; })
        .catch((e) => console.error('Waveform fetch failed:', e));
    }
  }, [selectedTrack]);

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
      <div className="waveform-panel">
        <canvas ref={canvasRef} id="waveform-canvas" />
        <div className="playhead-line" ref={playheadRef} />
        <div className="progress-track">
          <div className="progress-fill" ref={progressFillRef} />
        </div>
      </div>
    </div>
  );
}
