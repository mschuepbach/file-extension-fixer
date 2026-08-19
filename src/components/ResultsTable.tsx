import { useEffect, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { IconFile } from "@tabler/icons-react";
import type { Mismatch } from "../types";

interface Props {
  mismatches: Mismatch[];
  selected: Set<string>;
  onToggleRow: (index: number, shiftKey: boolean) => void;
  onToggleAll: () => void;
}

export function ResultsTable({ mismatches, selected, onToggleRow, onToggleAll }: Props) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: mismatches.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 41,
    overscan: 12,
  });

  const headerRef = useRef<HTMLInputElement>(null);
  const selectedCount = selected.size;
  const allSelected = mismatches.length > 0 && selectedCount === mismatches.length;
  const someSelected = selectedCount > 0 && !allSelected;

  useEffect(() => {
    if (headerRef.current) {
      headerRef.current.indeterminate = someSelected;
    }
  }, [someSelected]);

  if (mismatches.length === 0) {
    return (
      <div className="results-table">
        <div className="empty-state">No mismatches found yet.</div>
      </div>
    );
  }

  return (
    <div className="results-table">
      <div className="row-grid head">
        <input
          ref={headerRef}
          type="checkbox"
          checked={allSelected}
          onChange={onToggleAll}
          aria-label="Select all mismatches"
        />
        <span>
          File <span className="head-count">· {selectedCount} of {mismatches.length} selected</span>
        </span>
        <span>Current</span>
        <span />
        <span>Detected</span>
      </div>

      <div ref={parentRef} className="results-viewport">
        <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const mismatch = mismatches[virtualRow.index];
            const isSelected = selected.has(mismatch.path);

            return (
              <div
                key={mismatch.path}
                className={`row-grid results-row${isSelected ? " selected" : ""}`}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  height: virtualRow.size,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                onClick={(e) => onToggleRow(virtualRow.index, e.shiftKey)}
              >
                <input
                  type="checkbox"
                  checked={isSelected}
                  onChange={() => {}}
                  onClick={(e) => e.stopPropagation()}
                  onMouseDown={(e) => {
                    e.preventDefault();
                    onToggleRow(virtualRow.index, e.shiftKey);
                  }}
                />
                <span className="file-cell" title={mismatch.relativePath}>
                  <IconFile stroke={1.5} />
                  {mismatch.relativePath}
                </span>
                <span className="ext-cell">.{mismatch.currentExtension}</span>
                <span className="ext-cell">→</span>
                <span className="ext-badge">.{mismatch.detectedExtension}</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
