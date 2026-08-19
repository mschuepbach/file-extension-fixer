import { useEffect, useState } from "react";
import "./App.css";
import { SetupScreen } from "./screens/SetupScreen";
import { ResultsScreen } from "./screens/ResultsScreen";
import { DoneScreen } from "./screens/DoneScreen";
import { applyRenames, cancelScan, onMismatchFound, onScanProgress, scanFolder } from "./lib/tauri";
import type { ApplySummary, Mismatch } from "./types";

function App() {
  const [folder, setFolder] = useState<string | null>(null);
  const [mismatches, setMismatches] = useState<Mismatch[]>([]);
  const [totalScanned, setTotalScanned] = useState<number | null>(null);
  const [scanning, setScanning] = useState(false);
  const [applying, setApplying] = useState(false);
  const [doneSummary, setDoneSummary] = useState<ApplySummary | null>(null);
  const [remaining, setRemaining] = useState(0);

  useEffect(() => {
    // WebView2 shows its own browser-style context menu (Back, Reload,
    // Inspect...) by default in both dev and production builds. Rows
    // provide their own menu on right-click, so suppress the native one
    // everywhere else too rather than leave a mix of behaviors.
    function suppressNativeContextMenu(e: MouseEvent) {
      e.preventDefault();
    }
    window.addEventListener("contextmenu", suppressNativeContextMenu);
    return () => window.removeEventListener("contextmenu", suppressNativeContextMenu);
  }, []);

  async function startScan(folderPath: string) {
    setFolder(folderPath);
    setMismatches([]);
    setTotalScanned(null);
    setDoneSummary(null);
    setScanning(true);

    const unlistenMismatch = await onMismatchFound((mismatch) => {
      setMismatches((prev) => [...prev, mismatch]);
    });
    const unlistenProgress = await onScanProgress((progress) => {
      setTotalScanned(progress.scanned);
    });

    try {
      const summary = await scanFolder(folderPath);
      setTotalScanned(summary.totalScanned);
    } catch (err) {
      console.error("scan failed", err);
    } finally {
      unlistenMismatch();
      unlistenProgress();
      setScanning(false);
    }
  }

  function handleCancelScan() {
    cancelScan().catch((err) => console.error("cancel failed", err));
  }

  async function handleApply(selected: Mismatch[]) {
    setApplying(true);
    try {
      const summary = await applyRenames(
        selected.map((m) => ({ path: m.path, canonicalExtension: m.detectedExtension }))
      );
      setRemaining(mismatches.length - selected.length);
      setDoneSummary(summary);
    } catch (err) {
      console.error("apply failed", err);
    } finally {
      setApplying(false);
    }
  }

  function handleScanAnotherFolder() {
    setFolder(null);
    setMismatches([]);
    setTotalScanned(null);
    setDoneSummary(null);
  }

  if (folder === null) {
    return <SetupScreen onFolderChosen={startScan} />;
  }

  if (doneSummary !== null) {
    return (
      <DoneScreen
        summary={doneSummary}
        remaining={remaining}
        onScanAnotherFolder={handleScanAnotherFolder}
        onRescanFolder={() => startScan(folder)}
      />
    );
  }

  return (
    <ResultsScreen
      folder={folder}
      mismatches={mismatches}
      totalScanned={totalScanned}
      scanning={scanning}
      applying={applying}
      onChangeFolder={handleScanAnotherFolder}
      onRescan={() => startScan(folder)}
      onApply={handleApply}
      onCancelScan={handleCancelScan}
    />
  );
}

export default App;
