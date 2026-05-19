import { open } from '@tauri-apps/plugin-dialog';

interface TitlebarProps {
  onScanDirectory: (path: string) => void;
}

export default function Titlebar({ onScanDirectory }: TitlebarProps) {
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
