export interface Mismatch {
  path: string;
  relativePath: string;
  currentExtension: string;
  detectedExtension: string;
  /** Provisional - the filename this would be renamed to. */
  suggestedName: string;
  /** Provisional - whether suggestedName currently collides with an existing file. */
  hasConflict: boolean;
}

export interface ScanSummary {
  totalScanned: number;
  mismatchesFound: number;
  cancelled: boolean;
}

export interface ScanProgress {
  scanned: number;
}

export interface RenameOutcome {
  path: string;
  newPath: string | null;
  error: string | null;
}

export interface ApplySummary {
  renamed: number;
  failed: number;
}
