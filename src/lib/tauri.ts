import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
import type { ApplySummary, Mismatch, RenameOutcome, ScanProgress, ScanSummary } from "../types";

export { openPath, revealItemInDir };

export function pickFolder(): Promise<string | null> {
  return invoke("pick_folder");
}

export function scanFolder(folder: string): Promise<ScanSummary> {
  return invoke("scan_folder", { folder });
}

export function cancelScan(): Promise<void> {
  return invoke("cancel_scan");
}

export function onScanProgress(handler: (progress: ScanProgress) => void): Promise<UnlistenFn> {
  return listen<ScanProgress>("scan:progress", (event) => handler(event.payload));
}

export interface RenameRequest {
  path: string;
  canonicalExtension: string;
}

export function applyRenames(items: RenameRequest[]): Promise<ApplySummary> {
  return invoke("apply_renames", { items });
}

export function onMismatchesFound(handler: (mismatches: Mismatch[]) => void): Promise<UnlistenFn> {
  return listen<Mismatch[]>("scan:mismatches-found", (event) => handler(event.payload));
}

/** Fires once, after every mismatch batch has already been emitted - the
 * reliable signal that the event stream is done, since it travels the
 * same channel as the mismatch batches (unlike this command's own
 * invoke() return value, which is a separate, unordered channel). */
export function onScanComplete(handler: (summary: ScanSummary) => void): Promise<UnlistenFn> {
  return listen<ScanSummary>("scan:complete", (event) => handler(event.payload));
}

export function onApplyProgress(handler: (outcome: RenameOutcome) => void): Promise<UnlistenFn> {
  return listen<RenameOutcome>("apply:progress", (event) => handler(event.payload));
}
