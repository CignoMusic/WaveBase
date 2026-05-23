import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Titlebar from './components/Titlebar';
import Toolbar from './components/Toolbar';
import TrackList from './components/TrackList';
import PlayerBar from './components/PlayerBar';
import type { Track } from './components/TrackList';

interface ScannedFile {
  path: string;
  filename: string;
  extension: string;
  size_bytes: number;
  modified_at: string;
}

function App() {
  const [tracks, setTracks] = useState<Track[]>([]);
  const [activeFilter, setActiveFilter] = useState('All');
  const [selectedId, setSelectedId] = useState<string>('');

  const getFlatTracks = useCallback((): Track[] => {
    const flat: Track[] = [];
    for (const t of tracks) {
      flat.push(t);
      if (t.stems) flat.push(...t.stems);
    }
    return flat;
  }, [tracks]);

  const selectedTrack = getFlatTracks().find((t) => t.id === selectedId) ?? null;

  const handleScanDirectory = async (path: string) => {
    try {
      const files = await invoke<ScannedFile[]>('scan_directory', { path });
      const mapped: Track[] = files.map((f) => ({
        id: f.path,
        name: f.filename,
        path: f.path,
        bpm: null,
        key: '',
        artists: '',
        dotColor: 'green' as const,
      }));
      setTracks(mapped);
      if (mapped.length > 0) {
        setSelectedId(mapped[0].id);
      }
    } catch (err) {
      console.error('Scan failed:', err);
    }
  };

  const handlePlayTrack = useCallback(async (id: string) => {
    const flat = getFlatTracks();
    const track = flat.find((t) => t.id === id);
    if (!track) return;
    try {
      await invoke('play_audio', { path: track.path });
    } catch (e) {
      console.error('Play failed:', e);
    }
  }, [getFlatTracks]);

  const handleDoubleClickTrack = useCallback((id: string) => {
    setSelectedId(id);
    handlePlayTrack(id);
  }, [handlePlayTrack]);

  const handleNextTrack = useCallback(() => {
    const flat = getFlatTracks();
    if (flat.length === 0) return;
    const idx = flat.findIndex((t) => t.id === selectedId);
    const next = idx < flat.length - 1 ? flat[idx + 1] : flat[0];
    setSelectedId(next.id);
    handlePlayTrack(next.id);
  }, [getFlatTracks, selectedId, handlePlayTrack]);

  const handlePrevTrack = useCallback(() => {
    const flat = getFlatTracks();
    if (flat.length === 0) return;
    const idx = flat.findIndex((t) => t.id === selectedId);
    const prev = idx > 0 ? flat[idx - 1] : flat[flat.length - 1];
    setSelectedId(prev.id);
    handlePlayTrack(prev.id);
  }, [getFlatTracks, selectedId, handlePlayTrack]);

  const handleTogglePlay = useCallback(async () => {
    if (!selectedTrack) return;
    try {
      await invoke('toggle_playback', { path: selectedTrack.path });
    } catch (e) {
      console.error('Toggle playback failed:', e);
    }
  }, [selectedTrack]);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      switch (e.code) {
        case 'Space':
        case 'MediaPlayPause':
          e.preventDefault();
          handleTogglePlay();
          break;
        case 'MediaNextTrack':
          e.preventDefault();
          handleNextTrack();
          break;
        case 'MediaPreviousTrack':
          e.preventDefault();
          handlePrevTrack();
          break;
        case 'ArrowRight':
          if (e.ctrlKey) {
            e.preventDefault();
            handleNextTrack();
          }
          break;
        case 'ArrowLeft':
          if (e.ctrlKey) {
            e.preventDefault();
            handlePrevTrack();
          }
          break;
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [handleTogglePlay, handleNextTrack, handlePrevTrack]);

  return (
    <div className="app">
      <Titlebar onScanDirectory={handleScanDirectory} />
      <Toolbar
        activeFilter={activeFilter}
        onFilterChange={setActiveFilter}
      />
      <div className="main">
        <TrackList
          tracks={tracks}
          selectedId={selectedId}
          onSelect={setSelectedId}
          onDoubleClick={handleDoubleClickTrack}
        />
        <PlayerBar
          selectedTrack={selectedTrack}
          onNext={handleNextTrack}
          onPrev={handlePrevTrack}
        />
      </div>
    </div>
  );
}

export default App;
