import { IconArrowBackUp, IconCircleCheck } from "@tabler/icons-react";
import type { ApplySummary } from "../types";

interface Props {
  summary: ApplySummary;
  kind: "applied" | "undone";
  remaining: number;
  undoing: boolean;
  onScanAnotherFolder: () => void;
  onRescanFolder: () => void;
  onUndo: () => void;
}

export function DoneScreen({
  summary,
  kind,
  remaining,
  undoing,
  onScanAnotherFolder,
  onRescanFolder,
  onUndo,
}: Props) {
  const verb = kind === "applied" ? "renamed" : "restored";

  return (
    <div className="app-shell">
      <div className="centered">
        <div className="done-card">
          <IconCircleCheck stroke={1.5} aria-hidden="true" />
          <div className="headline">
            {summary.renamed} file{summary.renamed === 1 ? "" : "s"} {verb}
            {summary.failed > 0 ? `, ${summary.failed} failed` : ""}
          </div>
          {kind === "applied" && (
            <div className="subtext">
              {remaining > 0
                ? `${remaining} mismatch${remaining === 1 ? "" : "es"} left unselected`
                : "No mismatches left"}
            </div>
          )}
          <div className="actions">
            {kind === "applied" && summary.renamed > 0 && (
              <button onClick={onUndo} disabled={undoing}>
                <IconArrowBackUp size={16} stroke={1.5} aria-hidden="true" />
                {undoing ? "Undoing…" : "Undo"}
              </button>
            )}
            <button onClick={onScanAnotherFolder}>Scan another folder</button>
            <button className="primary" onClick={onRescanFolder}>
              Rescan this folder
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
