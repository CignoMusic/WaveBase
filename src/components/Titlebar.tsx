import { open } from '@tauri-apps/plugin-dialog';

interface TitlebarProps {
  onScanDirectory: (path: string) => void;
  onOpenSettings: () => void;
}

export default function Titlebar({ onScanDirectory, onOpenSettings }: TitlebarProps) {
  const handleScanClick = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select a directory to scan",
    });
    if (selected) {
      onScanDirectory(selected);
    }
  };

  return (
    <div className="titlebar">
      <div className="title-center">WaveBase</div>
      <div className="titlebar-actions">
        <button className="tool-btn" onClick={onOpenSettings} title="Manage tags">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <circle cx="6" cy="6" r="1.8" stroke="currentColor" strokeWidth="1.2" />
            <path d="M6 1.5v1M6 9.5v1M1.5 6h1M9.5 6h1" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
            <path d="M2.5 2.5l.7.7M8.8 8.8l.7.7M2.5 9.5l.7-.7M8.8 3.2l.7-.7" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </button>
        <button className="tool-btn" onClick={handleScanClick} title="Scan a directory for audio files">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M6 1v7M3 5l3 3 3-3" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M1 9v1a1 1 0 001 1h8a1 1 0 001-1V9" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
          Scan Directory
        </button>
      </div>
    </div>
  );
}
