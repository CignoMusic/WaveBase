import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { TagInfo, ScanRoot } from '../lib/ui-logic';

type SettingsTab = 'directories' | 'tags' | 'database';

interface SettingsPanelProps {
  tags: TagInfo[];
  onTagsChange: (tags: TagInfo[]) => void;
  onClose: () => void;
}

export default function SettingsPanel({ tags, onTagsChange, onClose }: SettingsPanelProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>('directories');

  const tabs: { key: SettingsTab; label: string }[] = [
    { key: 'directories', label: 'Directories' },
    { key: 'tags', label: 'Tags & Filtering' },
    { key: 'database', label: 'Database' },
  ];

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel settings-panel--wide" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="tool-btn" onClick={onClose}>
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M2 2l10 10M12 2L2 12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
        <div className="settings-body-h">
          <nav className="settings-sidebar">
            {tabs.map((t) => (
              <button
                key={t.key}
                className={`settings-tab${activeTab === t.key ? ' active' : ''}`}
                onClick={() => setActiveTab(t.key)}
              >
                {t.label}
              </button>
            ))}
          </nav>
          <div className="settings-content">
            {activeTab === 'directories' && <DirectoriesPanel />}
            {activeTab === 'tags' && <TagsPanel tags={tags} onTagsChange={onTagsChange} />}
            {activeTab === 'database' && <DatabasePanel />}
          </div>
        </div>
      </div>
    </div>
  );
}

/* ─── Directories ─── */

function DirectoriesPanel() {
  const [roots, setRoots] = useState<ScanRoot[]>([]);

  const loadRoots = useCallback(async () => {
    try {
      const r = await invoke<ScanRoot[]>('list_scan_roots');
      setRoots(r);
    } catch (e) {
      console.error('Failed to load scan roots:', e);
    }
  }, []);

  useEffect(() => { loadRoots(); }, [loadRoots]);

  const handleAdd = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Add directory to library',
    });
    if (!selected) return;
    try {
      await invoke('add_scan_root', { path: selected });
      await loadRoots();
    } catch (e) {
      console.error('Failed to add scan root:', e);
    }
  };

  const handleRemove = async (id: number, path: string) => {
    const ok = confirm(`Remove "${path}" from your library? Files will not be deleted.`);
    if (!ok) return;
    try {
      await invoke('remove_scan_root', { id });
      await loadRoots();
    } catch (e) {
      console.error('Failed to remove scan root:', e);
    }
  };

  return (
    <div className="settings-section">
      <h3>Scan Directories</h3>
      <p className="settings-desc">
        Add folders that WaveBase should watch for audio files.
      </p>
      <div className="dir-list">
        {roots.length === 0 && (
          <span className="settings-empty">No directories added yet.</span>
        )}
        {roots.map((root) => (
          <div key={root.id} className="dir-row">
            <span className="dir-path">{root.path}</span>
            <button
              className="tool-btn dir-remove-btn"
              onClick={() => handleRemove(root.id, root.path)}
              title="Remove from library"
            >
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M2 2l6 6M8 2L2 8" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
              </svg>
            </button>
          </div>
        ))}
      </div>
      <button className="tool-btn dir-add-btn" onClick={handleAdd}>
        <svg width="11" height="11" viewBox="0 0 11 11" fill="none">
          <path d="M5.5 1v9M1 5.5h9" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
        </svg>
        Add Directory
      </button>
    </div>
  );
}

/* ─── Tags & Filtering ─── */

function TagsPanel({ tags, onTagsChange }: { tags: TagInfo[]; onTagsChange: (t: TagInfo[]) => void }) {
  const [newTagName, setNewTagName] = useState('');
  const [fileCounts, setFileCounts] = useState<Record<number, number>>({});

  useEffect(() => {
    const counts: Record<number, number> = {};
    Promise.all(
      tags.map(async (tag) => {
        try {
          const c = await invoke<number>('get_tag_file_count', { tagId: tag.id });
          counts[tag.id] = c;
        } catch {
          counts[tag.id] = 0;
        }
      })
    ).then(() => setFileCounts(counts));
  }, [tags]);

  const handleCreateTag = useCallback(async () => {
    const name = newTagName.trim();
    if (!name) return;
    try {
      const tag = await invoke<TagInfo>('create_tag', { name });
      onTagsChange([...tags, tag]);
      setNewTagName('');
    } catch (e) {
      console.error('Create tag failed:', e);
    }
  }, [newTagName, tags, onTagsChange]);

  const handleDeleteTag = useCallback(async (tagId: number) => {
    const count = fileCounts[tagId] ?? 0;
    if (count > 0) {
      const ok = confirm(`Remove "${tags.find((t) => t.id === tagId)?.name}" from ${count} file(s)?`);
      if (!ok) return;
    }
    try {
      await invoke('delete_tag', { tagId });
      onTagsChange(tags.filter((t) => t.id !== tagId));
    } catch (e) {
      console.error('Delete tag failed:', e);
    }
  }, [tags, onTagsChange, fileCounts]);

  const handleTogglePin = useCallback(async (tagId: number) => {
    const tag = tags.find((t) => t.id === tagId);
    if (!tag) return;
    try {
      const updated = await invoke<TagInfo>('toggle_tag_pin', { tagId });
      onTagsChange(tags.map((t) => (t.id === tagId ? updated : t)));
    } catch (e) {
      console.error('Toggle pin failed:', e);
    }
  }, [tags, onTagsChange]);

  return (
    <div className="settings-section">
      <h3>Tags & Filtering</h3>
      <p className="settings-desc">
        Pinned tags appear as quick-filter buttons in the toolbar.
      </p>
      <div className="tag-mgmt-list">
        {tags.length === 0 && (
          <span className="settings-empty">No tags yet.</span>
        )}
        {tags.map((tag) => (
          <div key={tag.id} className="tag-mgmt-row">
            <div className="tag-mgmt-info">
              <span className="tag-mgmt-name">{tag.name}</span>
              <span className="tag-mgmt-count">{fileCounts[tag.id] ?? 0} file(s)</span>
            </div>
            <div className="tag-mgmt-actions">
              <button
                className={`tool-btn tag-pin-btn${tag.isPinned ? ' pinned' : ''}`}
                onClick={() => handleTogglePin(tag.id)}
                title={tag.isPinned ? 'Unpin from toolbar' : 'Pin to toolbar'}
              >
                {tag.isPinned ? 'Pinned' : 'Pin'}
              </button>
              {tag.isPreset ? (
                <span className="settings-preset-badge" title="Preset tag">Preset</span>
              ) : (
                <button
                  className="tool-btn tag-delete-btn"
                  onClick={() => handleDeleteTag(tag.id)}
                >
                  Delete
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
      <div className="tag-create-row">
        <input
          type="text"
          className="settings-input"
          placeholder="New tag name…"
          value={newTagName}
          onChange={(e) => setNewTagName(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') handleCreateTag(); }}
        />
        <button className="tool-btn" onClick={handleCreateTag} disabled={!newTagName.trim()}>
          Add Tag
        </button>
      </div>
    </div>
  );
}

/* ─── Database ─── */

function DatabasePanel() {
  const [dbSize, setDbSize] = useState<number | null>(null);
  const [confirmStep, setConfirmStep] = useState(0);

  const loadSize = useCallback(async () => {
    try {
      const bytes = await invoke<number>('get_database_size');
      setDbSize(bytes);
    } catch (e) {
      console.error('Failed to get DB size:', e);
    }
  }, []);

  useEffect(() => { loadSize(); }, [loadSize]);

  const formatSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const handleClearStart = () => {
    setConfirmStep(1);
  };

  const handleClearConfirm = () => {
    setConfirmStep(2);
  };

  const handleClearExecute = async () => {
    try {
      await invoke('clear_database');
      setDbSize(null);
      loadSize();
    } catch (e) {
      console.error('Failed to clear database:', e);
    }
    setConfirmStep(3);
  };

  const handleCancel = () => {
    setConfirmStep(0);
  };

  return (
    <div className="settings-section">
      <h3>Database</h3>
      <p className="settings-desc">
        View database storage usage and manage your library data.
      </p>

      <div className="db-info-row">
        <span className="db-info-label">Storage used</span>
        <span className="db-info-value">{dbSize !== null ? formatSize(dbSize) : '—'}</span>
      </div>

      {confirmStep === 0 && (
        <div className="db-clear-row">
          <button className="tool-btn db-clear-btn" onClick={handleClearStart}>
            Clear Database
          </button>
        </div>
      )}

      {confirmStep === 1 && (
        <div className="db-confirm">
          <p className="db-confirm-text">
            Are you sure you want to clear the database? This will remove all audio files and tags from the library. This cannot be undone.
          </p>
          <div className="db-confirm-actions">
            <button className="tool-btn" onClick={handleCancel}>Cancel</button>
            <button className="tool-btn db-clear-btn db-clear-btn--danger" onClick={handleClearConfirm}>
              Clear
            </button>
          </div>
        </div>
      )}

      {confirmStep === 2 && (
        <div className="db-confirm">
          <p className="db-confirm-text">
            Really? Are you absolutely sure? This will permanently delete all your library data.
          </p>
          <div className="db-confirm-actions">
            <button className="tool-btn" onClick={handleCancel}>Cancel</button>
            <button className="tool-btn db-clear-btn db-clear-btn--danger" onClick={handleClearExecute}>
              Clear
            </button>
          </div>
        </div>
      )}

      {confirmStep === 3 && (
        <div className="db-confirm">
          <p className="db-confirm-text db-confirm-text--done">
            All data has been cleared successfully.
          </p>
        </div>
      )}
    </div>
  );
}
