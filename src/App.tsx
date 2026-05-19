import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface ScannedFile {
  path: string;
  filename: string;
  extension: string;
  size_bytes: number;
  modified_at: string;
}

function formatSize(bytes: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let i = 0;
  let size = bytes;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

function App() {
  const [files, setFiles] = useState<ScannedFile[]>([]);
  const [loading, setLoading] = useState(false);

  async function handleScan() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select a directory to scan",
    });

    if (!selected) return;

    setLoading(true);
    try {
      const result = await invoke<ScannedFile[]>("scan_directory", {
        path: selected,
      });
      setFiles(result);
    } catch (err) {
      console.error("Scan failed:", err);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex h-screen w-screen flex-col bg-neutral-950 text-neutral-100">
      <header className="flex items-center justify-between border-b border-neutral-800 px-6 py-4">
        <h1 className="text-2xl font-bold tracking-tight">WaveBase</h1>
        <button
          onClick={handleScan}
          disabled={loading}
          className="rounded-lg bg-neutral-800 px-4 py-2 text-sm font-medium text-neutral-200 transition hover:bg-neutral-700 disabled:opacity-50"
        >
          {loading ? "Scanning..." : "Scan Directory"}
        </button>
      </header>

      <main className="flex-1 overflow-y-auto p-6">
        {files.length === 0 && !loading && (
          <div className="flex h-full items-center justify-center text-neutral-500">
            No files scanned yet. Click Scan Directory to get started.
          </div>
        )}

        {loading && (
          <div className="flex h-full items-center justify-center text-neutral-500">
            Scanning directory...
          </div>
        )}

        {files.length > 0 && !loading && (
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-neutral-800 text-neutral-400">
                <th className="pb-2 font-medium">Name</th>
                <th className="pb-2 font-medium">Type</th>
                <th className="pb-2 font-medium">Size</th>
              </tr>
            </thead>
            <tbody>
              {files.map((file) => (
                <tr
                  key={file.path}
                  className="border-b border-neutral-800/50 transition hover:bg-neutral-900"
                >
                  <td className="py-2 pr-4">{file.filename}</td>
                  <td className="py-2 pr-4 text-neutral-400">
                    {file.extension}
                  </td>
                  <td className="py-2 text-neutral-400">
                    {formatSize(file.size_bytes)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </main>
    </div>
  );
}

export default App;
