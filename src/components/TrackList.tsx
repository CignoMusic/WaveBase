import { useState, useRef, useEffect } from 'react';
import type { TagInfo } from '../lib/ui-logic';

export interface Track {
  id: string;
  name: string;
  path: string;
  bpm: number | null;
  key: string;
  artists: string;
  dotColor: 'green' | 'orange';
  stems?: Track[];
  tags?: TagInfo[];
  bpmAnalyzed?: boolean;
  keyAnalyzed?: boolean;
}

interface TrackListProps {
  tracks: Track[];
  selectedId: string;
  onSelect: (id: string) => void;
  onDoubleClick?: (id: string) => void;
  allTags: TagInfo[];
  onAddTag: (filePath: string, tagName: string) => void;
  onRemoveTag: (filePath: string, tagId: number) => void;
  onFilterByTag: (tagName: string) => void;
  onOpenSettings: () => void;
  fileTags: Record<string, TagInfo[]>;
}

export default function TrackList({
  tracks,
  selectedId,
  onSelect,
  onDoubleClick,
  allTags,
  onAddTag,
  onRemoveTag,
  onFilterByTag,
  onOpenSettings,
  fileTags,
}: TrackListProps) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [addTagFor, setAddTagFor] = useState<string | null>(null);
  const addTagRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (addTagRef.current && !addTagRef.current.contains(e.target as Node)) {
        setAddTagFor(null);
      }
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, []);

  const toggleExpanded = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const getTags = (track: Track): TagInfo[] => fileTags[track.path] ?? track.tags ?? [];

  return (
    <>
      <div className="col-header">
        <span />
        <span style={{ textAlign: 'center' }}>●</span>
        <span>Name</span>
        <span>BPM</span>
        <span>Key</span>
        <span>Artists</span>
        <span>Tags</span>
      </div>
      <div className="track-list">
        {tracks.map((track) => {
          const tags = getTags(track);
          return (
            <div key={track.id}>
              <div
                className={`row${selectedId === track.id ? ' selected' : ''}`}
                onClick={() => onSelect(track.id)}
                onDoubleClick={() => onDoubleClick?.(track.id)}
              >
                <div className="drag-handle">
                  <svg width="8" height="10" viewBox="0 0 8 10" fill="currentColor">
                    <circle cx="2" cy="2" r="1.1" />
                    <circle cx="6" cy="2" r="1.1" />
                    <circle cx="2" cy="5" r="1.1" />
                    <circle cx="6" cy="5" r="1.1" />
                    <circle cx="2" cy="8" r="1.1" />
                    <circle cx="6" cy="8" r="1.1" />
                  </svg>
                </div>
                <div className={`dot ${track.dotColor}`} />
                <div className="name-cell">
                  {track.stems && (
                    <div
                      className={`expand${expanded.has(track.id) ? ' open' : ''}`}
                      onClick={(e) => { e.stopPropagation(); toggleExpanded(track.id); }}
                    >
                      <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
                        <path d="M2 1.5L6 4L2 6.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
                      </svg>
                    </div>
                  )}
                  <span className="name-text">{track.name}</span>
                </div>
                <div className="bpm-cell">
                  {track.bpm ?? '—'}
                  {track.bpmAnalyzed && track.bpm != null && (
                    <span className="analyzed-hint" title="From audio analysis">~</span>
                  )}
                </div>
                <div className="key-cell">
                  {track.key}
                  {track.keyAnalyzed && track.key && (
                    <span className="analyzed-hint" title="From audio analysis">~</span>
                  )}
                </div>
                <div className="artists-cell">{track.artists}</div>
                <div className="tags-cell">
                  <div className="tag-list">
                    {tags.slice(0, 3).map((tag) => (
                      <span
                        key={tag.id}
                        className="tag-chip"
                        onClick={(e) => { e.stopPropagation(); onFilterByTag(tag.name); }}
                      >
                        {tag.name}
                        <span
                          className="tag-remove"
                          onClick={(e) => {
                            e.stopPropagation();
                            onRemoveTag(track.path, tag.id);
                          }}
                        >
                          ×
                        </span>
                      </span>
                    ))}
                    {tags.length > 3 && (
                      <span className="tag-more">+{tags.length - 3}</span>
                    )}
                    {addTagFor === track.path && (
                      <div className="tag-dropdown" ref={addTagRef}>
                        {allTags
                          .filter((t) => !tags.some((ft) => ft.id === t.id))
                          .map((tag) => (
                            <button
                              key={tag.id}
                              className="tag-dropdown-item"
                              onClick={() => {
                                onAddTag(track.path, tag.name);
                                setAddTagFor(null);
                              }}
                            >
                              {tag.name}
                            </button>
                          ))}
                        <div className="tag-dropdown-footer">
                          <button
                            className="tool-btn"
                            onClick={() => { setAddTagFor(null); onOpenSettings(); }}
                          >
                            Manage Tags…
                          </button>
                        </div>
                      </div>
                    )}
                  </div>
                  <button
                    className="tag-add-btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      setAddTagFor(addTagFor === track.path ? null : track.path);
                    }}
                  >
                    +
                  </button>
                </div>
              </div>
              {track.stems && (
                <div className={`stems-wrap${expanded.has(track.id) ? ' open' : ''}`}>
                  {track.stems.map((stem) => (
                    <div
                      key={stem.id}
                      className={`row stem${selectedId === stem.id ? ' selected' : ''}`}
                      onClick={() => onSelect(stem.id)}
                      onDoubleClick={() => onDoubleClick?.(stem.id)}
                    >
                      <div />
                      <div className="dot orange" style={{ width: 5, height: 5 }} />
                      <div className="name-cell">
                        <span className="name-text">{stem.name}</span>
                      </div>
                      <div className="bpm-cell">{stem.bpm ?? '—'}</div>
                      <div className="key-cell">{stem.key}</div>
                      <div className="artists-cell">{stem.artists}</div>
                      <div className="tags-cell" />
                    </div>
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </>
  );
}
