import { useState } from 'react';

export interface Track {
  id: string;
  name: string;
  bpm: number | null;
  key: string;
  artists: string;
  dotColor: 'green' | 'orange';
  stems?: Track[];
}

interface TrackListProps {
  tracks: Track[];
  selectedId: string;
  onSelect: (id: string) => void;
}

export default function TrackList({ tracks, selectedId, onSelect }: TrackListProps) {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const toggleExpanded = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  return (
    <>
      <div className="col-header">
        <span />
        <span style={{ textAlign: 'center' }}>●</span>
        <span>Name</span>
        <span>BPM</span>
        <span>Key</span>
        <span>Artists</span>
      </div>
      <div className="track-list">
        {tracks.map((track) => (
          <div key={track.id}>
            <div
              className={`row${selectedId === track.id ? ' selected' : ''}`}
              onClick={() => onSelect(track.id)}
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
              <div className="bpm-cell">{track.bpm ?? '—'}</div>
              <div className="key-cell">{track.key}</div>
              <div className="artists-cell">{track.artists}</div>
            </div>
            {track.stems && (
              <div className={`stems-wrap${expanded.has(track.id) ? ' open' : ''}`}>
                {track.stems.map((stem) => (
                  <div
                    key={stem.id}
                    className={`row stem${selectedId === stem.id ? ' selected' : ''}`}
                    onClick={() => onSelect(stem.id)}
                  >
                    <div />
                    <div className="dot orange" style={{ width: 5, height: 5 }} />
                    <div className="name-cell">
                      <span className="name-text">{stem.name}</span>
                    </div>
                    <div className="bpm-cell">{stem.bpm ?? '—'}</div>
                    <div className="key-cell">{stem.key}</div>
                    <div className="artists-cell">{stem.artists}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </>
  );
}
