import { useEffect, useMemo, useState } from "react";
import { IconAlertTriangle, IconChevronLeft, IconFolder, IconRefresh, IconX } from "@tabler/icons-react";
import type { Mismatch } from "../types";
import { ResultsTable } from "../components/ResultsTable";

interface Props {
  folder: string;
  mismatches: Mismatch[];
  totalScanned: number | null;
  scanning: boolean;
  scanCancelled: boolean;
  applying: boolean;
  applyProgress: { done: number; total: number } | null;
  onChangeFolder: () => void;
  onRescan: () => void;
  onApply: (selected: Mismatch[]) => void;
  onCancelScan: () => void;
}

export function ResultsScreen({
  folder,
  mismatches,
  totalScanned,
  scanning,
  scanCancelled,
  applying,
  applyProgress,
  onChangeFolder,
  onRescan,
  onApply,
  onCancelScan,
}: Props) {
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [lastIndex, setLastIndex] = useState<number | null>(null);

  function toggleRow(index: number, shiftKey: boolean) {
    const path = mismatches[index].path;

    setSelected((prev) => {
      const next = new Set(prev);

      if (shiftKey && lastIndex !== null) {
        const [start, end] = [lastIndex, index].sort((a, b) => a - b);
        const shouldSelect = !next.has(path);
        for (let i = start; i <= end; i++) {
          const p = mismatches[i].path;
          if (shouldSelect) {
            next.add(p);
          } else {
            next.delete(p);
          }
        }
      } else {
        if (next.has(path)) {
          next.delete(path);
        } else {
          next.add(path);
        }
      }

      return next;
    });

    setLastIndex(index);
  }

  function toggleAll() {
    setSelected((prev) =>
      prev.size === mismatches.length ? new Set() : new Set(mismatches.map((m) => m.path))
    );
  }

  const selectedMismatches = useMemo(
    () => mismatches.filter((m) => selected.has(m.path)),
    [mismatches, selected]
  );

  // Keyboard shortcuts: Ctrl/Cmd+A selects all, Enter applies the
  // current selection, Escape clears it. Skipped when an interactive
  // element already has focus so it can handle its own keys normally.
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const target = e.target as HTMLElement | null;
      if (target && ["INPUT", "TEXTAREA", "BUTTON", "SELECT"].includes(target.tagName)) {
        return;
      }

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") {
        e.preventDefault();
        setSelected(new Set(mismatches.map((m) => m.path)));
      } else if (e.key === "Escape") {
        setSelected(new Set());
      } else if (e.key === "Enter" && selectedMismatches.length > 0 && !applying) {
        e.preventDefault();
        onApply(selectedMismatches);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [mismatches, selectedMismatches, applying, onApply]);

  return (
    <div className="app-shell">
      <div className="results-header">
        <div>
          <div className="label">
            {scanning ? "Scanning" : scanCancelled ? "Scan cancelled" : "Scanned"}
          </div>
          <div className="folder">
            <IconFolder size={16} stroke={1.5} />
            {folder}
            {!scanning && scanCancelled && (
              <span className="cancelled-badge" title="Partial results - the scan was stopped early">
                <IconAlertTriangle size={13} stroke={1.5} />
                Partial results
              </span>
            )}
          </div>
        </div>
        <div className="stack-gap">
          <button onClick={onChangeFolder} disabled={scanning || applying}>
            <IconChevronLeft size={16} stroke={1.5} />
            Change folder
          </button>
          <button onClick={onRescan} disabled={scanning || applying}>
            <IconRefresh size={16} stroke={1.5} />
            Rescan
          </button>
        </div>
      </div>

      <div className="stat-grid">
        <div className="stat-card">
          <div className="label">Files scanned</div>
          <div className="value">{totalScanned ?? "—"}</div>
        </div>
        <div className="stat-card">
          <div className="label">Mismatches found</div>
          <div className="value danger">{mismatches.length}</div>
        </div>
      </div>

      <ResultsTable
        mismatches={mismatches}
        selected={selected}
        onToggleRow={toggleRow}
        onToggleAll={toggleAll}
      />

      <div className="apply-bar">
        {scanning && (
          <span className="scan-status">
            <span className="spinner" />
            Scanning…
          </span>
        )}
        {scanning && (
          <button onClick={onCancelScan}>
            <IconX size={16} stroke={1.5} />
            Cancel
          </button>
        )}
        {applying && applyProgress && (
          <span className="scan-status">
            <span className="spinner" />
            Applying… {applyProgress.done} of {applyProgress.total}
          </span>
        )}
        <button
          className="primary"
          disabled={selectedMismatches.length === 0 || applying}
          onClick={() => onApply(selectedMismatches)}
        >
          Apply {selectedMismatches.length} change{selectedMismatches.length === 1 ? "" : "s"}
        </button>
      </div>
    </div>
  );
}
