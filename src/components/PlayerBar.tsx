import { useRef, useEffect, useState, useCallback } from 'react';
import { formatTime, generateWaveformData, drawWaveformToCanvas } from '../lib/ui-logic';
import type { Track } from './TrackList';

interface PlayerBarProps {
  selectedTrack: Track | null;
}

export default function PlayerBar({ selectedTrack }: PlayerBarProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const playheadRef = useRef<HTMLDivElement>(null);
  const progressFillRef = useRef<HTMLDivElement>(null);
  const rafRef = useRef(0);
  const lastTsRef = useRef(0);
  const waveRef = useRef<number[] | null>(null);
  const progressRef = useRef(0);
  const [playing, setPlaying] = useState(false);
  const [progress, setProgress] = useState(0);
  const [duration] = useState(154);

  progressRef.current = progress;

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const wave = waveRef.current ?? generateWaveformData(200);
    waveRef.current = wave;
    drawWaveformToCanvas(canvas, wave, progress);

    const parent = canvas.parentElement;
    if (playheadRef.current && parent) {
      const w = parent.getBoundingClientRect().width - 24;
      playheadRef.current.style.left = `${12 + w * progress}px`;
    }
    if (progressFillRef.current) {
      progressFillRef.current.style.width = `${progress * 100}%`;
    }
  }, [progress]);

  useEffect(() => {
    draw();
  });

  useEffect(() => {
    if (!playing) return;
    lastTsRef.current = 0;
    const animate = (ts: number) => {
      if (!lastTsRef.current) lastTsRef.current = ts;
      const dt = (ts - lastTsRef.current) / 1000;
      lastTsRef.current = ts;
      const newProgress = Math.min(1, progressRef.current + dt / duration);
      setProgress(newProgress >= 1 ? 0 : newProgress);
      rafRef.current = requestAnimationFrame(animate);
    };
    rafRef.current = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(rafRef.current);
  }, [playing, duration]);

  useEffect(() => {
    setProgress(0);
    waveRef.current = null;
    lastTsRef.current = 0;
  }, [selectedTrack]);

  useEffect(() => {
    const onResize = () => {
      waveRef.current = null;
      if (canvasRef.current) {
        const wave = generateWaveformData(200);
        waveRef.current = wave;
        drawWaveformToCanvas(canvasRef.current, wave, progressRef.current);
      }
    };
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  const togglePlay = () => {
    if (playing) {
      cancelAnimationFrame(rafRef.current);
    }
    lastTsRef.current = 0;
    setPlaying((p) => !p);
  };

  const skipBack = () => {
    setProgress(0);
    lastTsRef.current = 0;
  };

  const seekPanel = (e: React.MouseEvent) => {
    const panel = e.currentTarget as HTMLElement;
    const rect = panel.getBoundingClientRect();
    const relX = e.clientX - rect.left - 12;
    const w = rect.width - 24;
    setProgress(Math.max(0, Math.min(1, relX / w)));
    lastTsRef.current = 0;
  };

  const time = formatTime(progress * duration);
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
          <button className="t-btn" onClick={skipBack} title="Skip to start">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <rect x="1.5" y="2" width="2" height="10" rx="0.5" />
              <path d="M12.5 2L5.5 7L12.5 12V2Z" />
            </svg>
          </button>
          <button className="t-btn play-pause" onClick={togglePlay} title={playing ? 'Pause' : 'Play'}>
            {playing ? (
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
          <button className="t-btn" title="Skip forward">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <rect x="10.5" y="2" width="2" height="10" rx="0.5" />
              <path d="M1.5 2L8.5 7L1.5 12V2Z" />
            </svg>
          </button>
        </div>
        <div className="sep" />
        <div className="time-counter">{time}</div>
        <div className="vol-row">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M3 5H1v4h2l4 3V2L3 5Z" fill="currentColor" />
            <path d="M10 4.5c.8.6 1.3 1.5 1.3 2.5s-.5 1.9-1.3 2.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
            <path d="M12 2.5c1.2 1.1 1.8 2.7 1.8 4.5s-.6 3.4-1.8 4.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
          <input type="range" className="vol-slider" min="0" max="100" defaultValue="80" />
        </div>
      </div>
      <div className="waveform-panel" onClick={seekPanel}>
        <canvas ref={canvasRef} id="waveform-canvas" />
        <div className="playhead-line" ref={playheadRef} />
        <div className="progress-track">
          <div className="progress-fill" ref={progressFillRef} />
        </div>
      </div>
    </div>
  );
}
