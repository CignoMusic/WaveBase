import { useRef } from 'react';

interface TitlebarProps {
  onScanDirectory: (path: string) => void;
}

export default function Titlebar({ onScanDirectory }: TitlebarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  const handleScanClick = () => {
    inputRef.current?.click();
  };

  const handleDirChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;

    const file = files[0] as File & { path?: string };
    const fullPath = file.path || file.webkitRelativePath;
    if (fullPath) {
      const dir = fullPath.substring(0, fullPath.lastIndexOf('\\'));
      onScanDirectory(dir);
    }
    e.target.value = '';
  };

  return (
    <div className="titlebar">
      <div className="title-center">WaveBase</div>
      <div className="titlebar-actions">
        <input
          ref={inputRef}
          type="file"
          style={{ display: 'none' }}
          onChange={handleDirChange}
          {...{ webkitdirectory: '', directory: '' } as React.InputHTMLAttributes<HTMLInputElement>}
        />
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
