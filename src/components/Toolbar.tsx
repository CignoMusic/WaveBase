import { useState, useRef, useEffect } from 'react';
import type { TagInfo, TagProgress } from '../lib/ui-logic';

interface ToolbarProps {
  pinnedTags: TagInfo[];
  allTags: TagInfo[];
  activeTagNames: string[];
  onTagToggle: (tagName: string) => void;
  tagProgress: TagProgress | null;
  onOpenSettings: () => void;
  searchQuery: string;
  onSearchChange: (q: string) => void;
}

export default function Toolbar({
  pinnedTags,
  allTags,
  activeTagNames,
  onTagToggle,
  tagProgress,
  onOpenSettings,
  searchQuery,
  onSearchChange,
}: ToolbarProps) {
  const [showFilter, setShowFilter] = useState(false);
  const filterRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (filterRef.current && !filterRef.current.contains(e.target as Node)) {
        setShowFilter(false);
      }
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, []);

  const hasActiveFilter = activeTagNames.length > 0;

  return (
    <div className="toolbar">
      <button
        className={`tool-btn${activeTagNames.length === 0 ? ' active' : ''}`}
        onClick={() => {
          if (activeTagNames.length > 0) {
            activeTagNames.forEach((t) => onTagToggle(t));
          }
        }}
      >
        All
      </button>
      {pinnedTags.map((tag) => (
        <button
          key={tag.id}
          className={`tool-btn${activeTagNames.includes(tag.name) ? ' active' : ''}`}
          onClick={() => onTagToggle(tag.name)}
        >
          {tag.name}
        </button>
      ))}
      <div className="toolbar-sep" />
      <div className="filter-wrap" ref={filterRef}>
        <button
          className={`tool-btn filter-btn${hasActiveFilter ? ' active' : ''}`}
          onClick={() => setShowFilter(!showFilter)}
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M1 3h10M3 6h6M5 9h2" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
          Filter
          {hasActiveFilter && ` (${activeTagNames.length})`}
        </button>
        {showFilter && (
          <div className="filter-dropdown">
            {allTags.map((tag) => (
              <label key={tag.id} className="filter-option">
                <input
                  type="checkbox"
                  checked={activeTagNames.includes(tag.name)}
                  onChange={() => onTagToggle(tag.name)}
                />
                <span>{tag.name}</span>
              </label>
            ))}
            <div className="filter-dropdown-footer">
              <button className="tool-btn" onClick={onOpenSettings}>
                Manage Tags…
              </button>
            </div>
          </div>
        )}
      </div>
      {tagProgress?.status === 'scanning' && (
        <div className="tag-progress">
          <div className="tag-progress-bar">
            <div
              className="tag-progress-fill"
              style={{
                width: `${tagProgress.total > 0 ? (tagProgress.processed / tagProgress.total) * 100 : 0}%`,
              }}
            />
          </div>
          <span className="tag-progress-text">
            Tagging: {tagProgress.processed}/{tagProgress.total}
          </span>
        </div>
      )}
      {tagProgress?.status === 'complete' && (
        <div className="tag-progress tag-progress-done">
          <span className="tag-progress-text">✓ Tagged</span>
        </div>
      )}
      <div className="search-box">
        <svg width="11" height="11" viewBox="0 0 11 11" fill="none">
          <circle cx="4.5" cy="4.5" r="3.5" stroke="currentColor" strokeWidth="1.2" />
          <path d="M7.5 7.5l2 2" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
        </svg>
        <input
          type="text"
          placeholder="Search library…"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
        />
      </div>
    </div>
  );
}
