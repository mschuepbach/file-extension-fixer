import { useEffect, useState } from "react";
import "./App.css";
import { SetupScreen } from "./screens/SetupScreen";
import { ResultsScreen } from "./screens/ResultsScreen";
import { DoneScreen } from "./screens/DoneScreen";
import {
  applyRenames,
  cancelScan,
  onApplyComplete,
  onApplyProgress,
  onMismatchesFound,
  onScanComplete,
  onScanProgress,
  scanFolder,
  undoLastApply,
} from "./lib/tauri";
import type { ApplySummary, Mismatch } from "./types";

function App() {
  const [folder, setFolder] = useState<string | null>(null);
  const [mismatches, setMismatches] = useState<Mismatch[]>([]);
  const [totalScanned, setTotalScanned] = useState<number | null>(null);
  const [scanning, setScanning] = useState(false);
  const [applying, setApplying] = useState(false);
  const [applyProgress, setApplyProgress] = useState<{ done: number; total: number } | null>(null);
  const [doneSummary, setDoneSummary] = useState<ApplySummary | null>(null);
  const [doneKind, setDoneKind] = useState<"applied" | "undone">("applied");
  const [undoing, setUndoing] = useState(false);
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

    let resolveComplete: () => void;
    const completePromise = new Promise<void>((resolve) => {
      resolveComplete = resolve;
    });

    const unlistenMismatches = await onMismatchesFound((batch) => {
      setMismatches((prev) => [...prev, ...batch]);
    });
    const unlistenProgress = await onScanProgress((progress) => {
      setTotalScanned(progress.scanned);
    });
    const unlistenComplete = await onScanComplete((summary) => {
      setTotalScanned(summary.totalScanned);
      resolveComplete();
    });

    try {
      await scanFolder(folderPath);
      // scanFolder()'s invoke() resolving isn't a reliable "every
      // mismatch batch has arrived" signal - invoke and emit/listen are
      // separate channels with no ordering guarantee between them, so
      // wait for the scan:complete event itself instead.
      await completePromise;
    } catch (err) {
      console.error("scan failed", err);
    } finally {
      unlistenMismatches();
      unlistenProgress();
      unlistenComplete();
      setScanning(false);
    }
  }

  function handleCancelScan() {
    cancelScan().catch((err) => console.error("cancel failed", err));
  }

  async function handleApply(selected: Mismatch[]) {
    setApplying(true);
    setApplyProgress({ done: 0, total: selected.length });

    let resolveComplete: (summary: ApplySummary) => void;
    const completePromise = new Promise<ApplySummary>((resolve) => {
      resolveComplete = resolve;
    });

    const unlistenProgress = await onApplyProgress((batch) => {
      setApplyProgress((prev) => (prev ? { ...prev, done: prev.done + batch.length } : prev));
    });
    const unlistenComplete = await onApplyComplete((summary) => resolveComplete(summary));

    try {
      const items = selected.map((m) => ({ path: m.path, canonicalExtension: m.detectedExtension }));
      await applyRenames(items);
      const summary = await completePromise;
      setRemaining(mismatches.length - selected.length);
      setDoneKind("applied");
      setDoneSummary(summary);
    } catch (err) {
      console.error("apply failed", err);
    } finally {
      unlistenProgress();
      unlistenComplete();
      setApplying(false);
      setApplyProgress(null);
    }
  }

  async function handleUndo() {
    setUndoing(true);

    let resolveComplete: (summary: ApplySummary) => void;
    const completePromise = new Promise<ApplySummary>((resolve) => {
      resolveComplete = resolve;
    });
    const unlistenComplete = await onApplyComplete((summary) => resolveComplete(summary));

    try {
      await undoLastApply();
      const summary = await completePromise;
      setDoneKind("undone");
      setDoneSummary(summary);
    } catch (err) {
      console.error("undo failed", err);
    } finally {
      unlistenComplete();
      setUndoing(false);
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
        kind={doneKind}
        remaining={remaining}
        undoing={undoing}
        onScanAnotherFolder={handleScanAnotherFolder}
        onRescanFolder={() => startScan(folder)}
        onUndo={handleUndo}
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
      applyProgress={applyProgress}
      onChangeFolder={handleScanAnotherFolder}
      onRescan={() => startScan(folder)}
      onApply={handleApply}
      onCancelScan={handleCancelScan}
    />
  );
}

export default App;
