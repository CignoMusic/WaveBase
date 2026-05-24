import { useState, useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Titlebar from './components/Titlebar';
import Toolbar from './components/Toolbar';
import TrackList from './components/TrackList';
import PlayerBar from './components/PlayerBar';
import SettingsPanel from './components/SettingsPanel';
import type { Track } from './components/TrackList';
import type { TagInfo, TagProgress } from './lib/ui-logic';

interface ScannedFile {
  path: string;
  filename: string;
  extension: string;
  size_bytes: number;
  modified_at: string;
}

interface AudioFile {
  id: number;
  path: string;
  filename: string;
  folder_path: string;
  extension: string;
  size_bytes: number;
  modified_at: string;
  duration_secs: number | null;
  track_name: string | null;
  bpm: number | null;
  key: string | null;
  artist: string | null;
  bpm_analyzed: boolean;
  key_analyzed: boolean;
  created_at: string;
  updated_at: string;
}

function App() {
  const [tracks, setTracks] = useState<Track[]>([]);
  const [selectedId, setSelectedId] = useState<string>('');
  const [allTags, setAllTags] = useState<TagInfo[]>([]);
  const [activeTagNames, setActiveTagNames] = useState<string[]>([]);
  const [tagProgress, setTagProgress] = useState<TagProgress | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [fileTags, setFileTags] = useState<Record<string, TagInfo[]>>({});
  const progressPollRef = useRef<number>(0);
  const loadAllFileTagsRef = useRef<(paths: string[]) => Promise<void>>(async () => {});

  const getFlatTracks = useCallback((): Track[] => {
    const flat: Track[] = [];
    for (const t of tracks) {
      flat.push(t);
      if (t.stems) flat.push(...t.stems);
    }
    return flat;
  }, [tracks]);

  const selectedTrack = getFlatTracks().find((t) => t.id === selectedId) ?? null;

  // Attach tags to tracks for the filter
  const tracksWithTags: Track[] = tracks.map((t) => ({
    ...t,
    tags: fileTags[t.path] ?? t.tags,
  }));

  const pinnedTags = allTags.filter((t) => t.isPinned);

  const filteredTracks = activeTagNames.length === 0
    ? tracksWithTags
    : tracksWithTags.filter((track) =>
        activeTagNames.every((tagName) =>
          (track.tags ?? []).some((t) => t.name === tagName)
        )
      );

  const loadAllFileTags = useCallback(async (paths: string[]) => {
    const map: Record<string, TagInfo[]> = {};
    for (const p of paths) {
      try {
        map[p] = await invoke<TagInfo[]>('list_file_tags', { filePath: p });
      } catch {
        map[p] = [];
      }
    }
    setFileTags((prev) => ({ ...prev, ...map }));
  }, []);

  loadAllFileTagsRef.current = loadAllFileTags;

  // Fetch tags and files on mount
  useEffect(() => {
    invoke<TagInfo[]>('get_all_tags').then(setAllTags).catch(console.error);
    invoke<AudioFile[]>('list_files', { limit: 5000, offset: 0 })
      .then(async (files) => {
        const mapped: Track[] = files.map((f) => ({
          id: f.path,
          name: f.filename,
          path: f.path,
          bpm: f.bpm,
          key: f.key ?? '',
          artists: f.artist ?? '',
          bpmAnalyzed: f.bpm_analyzed,
          keyAnalyzed: f.key_analyzed,
          dotColor: (f.extension === 'wav' || f.extension === 'aiff' || f.extension === 'aif') ? 'green' as const : 'orange' as const,
        }));
        setTracks(mapped);
        if (mapped.length > 0) {
          setSelectedId(mapped[0].id);
        }
        await loadAllFileTags(mapped.map((t) => t.path));
      })
      .catch(console.error);
  }, [loadAllFileTags]);

  // Tag progress polling
  useEffect(() => {
    if (tagProgress?.status === 'complete' || tagProgress?.status === 'idle' || tagProgress === null) {
      return;
    }
    if (tagProgress?.status === 'scanning' && !progressPollRef.current) {
      progressPollRef.current = window.setInterval(async () => {
        try {
          const p = await invoke<TagProgress>('get_tag_progress');
          setTagProgress(p);
          if (p.status === 'complete') {
            clearInterval(progressPollRef.current);
            progressPollRef.current = 0;
            invoke<TagInfo[]>('get_all_tags').then(setAllTags).catch(console.error);
            invoke<AudioFile[]>('list_files', { limit: 5000, offset: 0 })
              .then(async (files) => {
                const mapped: Track[] = files.map((f) => ({
                  id: f.path,
                  name: f.filename,
                  path: f.path,
                  bpm: f.bpm,
                  key: f.key ?? '',
                  artists: f.artist ?? '',
                  bpmAnalyzed: f.bpm_analyzed,
                  keyAnalyzed: f.key_analyzed,
                  dotColor: (f.extension === 'wav' || f.extension === 'aiff' || f.extension === 'aif') ? 'green' as const : 'orange' as const,
                }));
                setTracks(mapped);
                loadAllFileTagsRef.current(mapped.map((t) => t.path));
              })
              .catch(console.error);
          }
        } catch (e) {
          console.error('Tag progress poll failed:', e);
        }
      }, 300);
    }
    return () => {
      if (progressPollRef.current) {
        clearInterval(progressPollRef.current);
        progressPollRef.current = 0;
      }
    };
  }, [tagProgress?.status]);

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
        bpmAnalyzed: false,
        keyAnalyzed: false,
        dotColor: 'green' as const,
      }));
      setTracks(mapped);
      if (mapped.length > 0) {
        setSelectedId(mapped[0].id);
      }
      setTagProgress({ total: mapped.length, processed: 0, status: 'scanning' });
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

  const handleTagToggle = useCallback((tagName: string) => {
    setActiveTagNames((prev) =>
      prev.includes(tagName) ? prev.filter((t) => t !== tagName) : [...prev, tagName]
    );
  }, []);

  const handleAddTag = useCallback(async (filePath: string, tagName: string) => {
    try {
      const tag = await invoke<TagInfo>('add_tag', { filePath, tagName });
      setFileTags((prev) => {
        const existing = prev[filePath] ?? [];
        if (existing.some((t) => t.id === tag.id)) return prev;
        return { ...prev, [filePath]: [...existing, tag] };
      });
    } catch (e) {
      console.error('Add tag failed:', e);
    }
  }, []);

  const handleRemoveTag = useCallback(async (filePath: string, tagId: number) => {
    try {
      await invoke('remove_tag', { filePath, tagId });
      setFileTags((prev) => ({
        ...prev,
        [filePath]: (prev[filePath] ?? []).filter((t) => t.id !== tagId),
      }));
    } catch (e) {
      console.error('Remove tag failed:', e);
    }
  }, []);

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
      <Titlebar onScanDirectory={handleScanDirectory} onOpenSettings={() => setShowSettings(true)} />
      <Toolbar
        pinnedTags={pinnedTags}
        allTags={allTags}
        activeTagNames={activeTagNames}
        onTagToggle={handleTagToggle}
        tagProgress={tagProgress}
        onOpenSettings={() => setShowSettings(true)}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
      />
      <div className="main">
        <TrackList
          tracks={filteredTracks}
          selectedId={selectedId}
          onSelect={setSelectedId}
          onDoubleClick={handleDoubleClickTrack}
          allTags={allTags}
          onAddTag={handleAddTag}
          onRemoveTag={handleRemoveTag}
          onFilterByTag={handleTagToggle}
          onOpenSettings={() => setShowSettings(true)}
          fileTags={fileTags}
        />
        <PlayerBar
          selectedTrack={selectedTrack}
          onNext={handleNextTrack}
          onPrev={handlePrevTrack}
        />
      </div>
      {showSettings && (
        <SettingsPanel
          tags={allTags}
          onTagsChange={setAllTags}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}

export default App;
