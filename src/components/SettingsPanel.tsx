import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { TagInfo } from '../lib/ui-logic';

interface SettingsPanelProps {
  tags: TagInfo[];
  onTagsChange: (tags: TagInfo[]) => void;
  onClose: () => void;
}

export default function SettingsPanel({ tags, onTagsChange, onClose }: SettingsPanelProps) {
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
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Tag Manager</h2>
          <button className="tool-btn" onClick={onClose}>
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M2 2l10 10M12 2L2 12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
        <div className="settings-body">
          {tags.map((tag) => (
            <div key={tag.id} className="settings-row">
              <div className="settings-row-info">
                <span className="settings-tag-name">{tag.name}</span>
                <span className="settings-tag-count">{fileCounts[tag.id] ?? 0} file(s)</span>
              </div>
              <div className="settings-row-actions">
                <button
                  className={`tool-btn settings-pin-btn${tag.isPinned ? ' pinned' : ''}`}
                  onClick={() => handleTogglePin(tag.id)}
                  title={tag.isPinned ? 'Unpin from toolbar' : 'Pin to toolbar'}
                >
                  {tag.isPinned ? 'Pinned' : 'Pin'}
                </button>
                {tag.isPreset ? (
                  <span className="settings-preset-badge" title="Preset tag">Preset</span>
                ) : (
                  <button
                    className="tool-btn settings-delete-btn"
                    onClick={() => handleDeleteTag(tag.id)}
                  >
                    Delete
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
        <div className="settings-footer">
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
    </div>
  );
}
