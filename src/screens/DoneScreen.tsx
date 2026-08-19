import { IconCircleCheck } from "@tabler/icons-react";
import type { ApplySummary } from "../types";

interface Props {
  summary: ApplySummary;
  remaining: number;
  onScanAnotherFolder: () => void;
  onRescanFolder: () => void;
}

export function DoneScreen({ summary, remaining, onScanAnotherFolder, onRescanFolder }: Props) {
  return (
    <div className="app-shell">
      <div className="centered">
        <div className="done-card">
          <IconCircleCheck stroke={1.5} />
          <div className="headline">
            {summary.renamed} file{summary.renamed === 1 ? "" : "s"} renamed
            {summary.failed > 0 ? `, ${summary.failed} failed` : ""}
          </div>
          <div className="subtext">
            {remaining > 0
              ? `${remaining} mismatch${remaining === 1 ? "" : "es"} left unselected`
              : "No mismatches left"}
          </div>
          <div className="actions">
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
