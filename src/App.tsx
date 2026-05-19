import { useState, useCallback } from 'react';
import Titlebar from './components/Titlebar';
import Toolbar from './components/Toolbar';
import TrackList from './components/TrackList';
import PlayerBar from './components/PlayerBar';
import type { Track } from './components/TrackList';

const MOCK_TRACKS: Track[] = [
  {
    id: 'beat-001',
    name: 'Beat 001',
    bpm: 123,
    key: 'C min',
    artists: 'Artist 1, Artist 2',
    dotColor: 'green',
  },
  {
    id: 'beat-002',
    name: 'Beat 002',
    bpm: 132,
    key: 'D min',
    artists: 'Artist 1, Artist 3',
    dotColor: 'orange',
  },
  {
    id: 'loop-001',
    name: 'Loop 001',
    bpm: 90,
    key: 'F# min',
    artists: 'Artist 1',
    dotColor: 'orange',
    stems: [
      {
        id: 'loop-001-stem-1',
        name: 'Stem 1',
        bpm: 90,
        key: 'F# min',
        artists: 'Artist 1',
        dotColor: 'orange',
      },
      {
        id: 'loop-001-stem-2',
        name: 'Stem 2',
        bpm: 90,
        key: 'F# min',
        artists: 'Artist 1',
        dotColor: 'orange',
      },
      {
        id: 'loop-001-stem-3',
        name: 'Stem 3',
        bpm: 90,
        key: 'F# min',
        artists: 'Artist 1',
        dotColor: 'orange',
      },
    ],
  },
];

function App() {
  const [activeFilter, setActiveFilter] = useState('All');
  const [selectedId, setSelectedId] = useState(MOCK_TRACKS[0].id);

  const getFlatTracks = useCallback((): Track[] => {
    const flat: Track[] = [];
    for (const t of MOCK_TRACKS) {
      flat.push(t);
      if (t.stems) flat.push(...t.stems);
    }
    return flat;
  }, []);

  const selectedTrack = getFlatTracks().find((t) => t.id === selectedId) ?? null;

  const handleScanDirectory = (path: string) => {
    console.log('Scan directory selected:', path);
    // TODO: Wire to scan_directory Tauri command from useFileScanner stub
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
          tracks={MOCK_TRACKS}
          selectedId={selectedId}
          onSelect={setSelectedId}
        />
        <PlayerBar selectedTrack={selectedTrack} />
      </div>
    </div>
  );
}

export default App;
