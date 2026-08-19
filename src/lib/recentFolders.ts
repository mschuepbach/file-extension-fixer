const STORAGE_KEY = "recentFolders";
const MAX_ENTRIES = 5;

export function getRecentFolders(): string[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.filter((f) => typeof f === "string") : [];
  } catch {
    return [];
  }
}

export function addRecentFolder(folder: string): void {
  const updated = [folder, ...getRecentFolders().filter((f) => f !== folder)].slice(0, MAX_ENTRIES);
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
  } catch {
    // localStorage unavailable (e.g. disabled) - recent folders just won't persist.
  }
}

export function folderName(path: string): string {
  const segments = path.split(/[\\/]/).filter(Boolean);
  return segments[segments.length - 1] ?? path;
}
