import { useState } from 'react';

const FILTERS = ['All', 'Beats', 'Loops', 'Stems'];

interface ToolbarProps {
  activeFilter: string;
  onFilterChange: (filter: string) => void;
}

export default function Toolbar({ activeFilter, onFilterChange }: ToolbarProps) {
  const [query, setQuery] = useState('');

  return (
    <div className="toolbar">
      {FILTERS.map((f) => (
        <button
          key={f}
          className={`tool-btn${activeFilter === f ? ' active' : ''}`}
          onClick={() => onFilterChange(f)}
        >
          {f}
        </button>
      ))}
      <div className="toolbar-sep" />
      <button className="tool-btn">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M1 3h10M3 6h6M5 9h2" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
        </svg>
        Filter
      </button>
      <div className="search-box">
        <svg width="11" height="11" viewBox="0 0 11 11" fill="none">
          <circle cx="4.5" cy="4.5" r="3.5" stroke="currentColor" strokeWidth="1.2" />
          <path d="M7.5 7.5l2 2" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
        </svg>
        <input
          type="text"
          placeholder="Search library…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
      </div>
    </div>
  );
}
