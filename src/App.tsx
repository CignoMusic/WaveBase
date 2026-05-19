import { useState, useCallback } from 'react';
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
        />
        <PlayerBar selectedTrack={selectedTrack} />
      </div>
    </div>
  );
}

export default App;
