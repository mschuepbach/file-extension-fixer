import { useEffect, useState } from "react";
import { IconFolderPlus } from "@tabler/icons-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { pickFolder } from "../lib/tauri";

interface Props {
  onFolderChosen: (folder: string) => void;
}

export function SetupScreen({ onFolderChosen }: Props) {
  const [dragActive, setDragActive] = useState(false);

  useEffect(() => {
    // Native OS drag events, not HTML5 DnD - the webview intercepts the
    // browser-level events, so highlighting and drop handling both go
    // through this single Tauri event instead.
    const unlistenPromise = getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        setDragActive(true);
      } else if (event.payload.type === "drop") {
        setDragActive(false);
        if (event.payload.paths.length > 0) {
          onFolderChosen(event.payload.paths[0]);
        }
      } else {
        setDragActive(false);
      }
    });

    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, [onFolderChosen]);

  async function handleBrowse() {
    const folder = await pickFolder();
    if (folder) {
      onFolderChosen(folder);
    }
  }

  return (
    <div className="app-shell">
      <div className="centered">
        <div className={`dropzone${dragActive ? " drag-active" : ""}`}>
          <IconFolderPlus className="dropzone-icon" stroke={1.5} />
          <div className="dropzone-title">Drag a folder here</div>
          <div className="dropzone-or">or</div>
          <button className="primary" onClick={handleBrowse}>
            Browse for folder
          </button>
        </div>
        <div className="dropzone-hint">
          Scans subdirectories automatically for common photo, video and audio files
        </div>
      </div>
    </div>
  );
}
