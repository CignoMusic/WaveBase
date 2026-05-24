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
        <button className="tool-btn" onClick={onOpenSettings} title="Settings">
          <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
            <circle cx="7" cy="7" r="2.5" stroke="currentColor" strokeWidth="1.2" />
            <path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
            <path d="M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
            <path d="M3.8 3.8a4.5 4.5 0 01.7-.6M9.5 9.5a4.5 4.5 0 01.6.7M3.8 10.2a4.5 4.5 0 01-.6-.7M10.2 3.8a4.5 4.5 0 01.6-.7" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
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
