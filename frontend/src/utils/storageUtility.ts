import type { SavedWorkspaceState } from "../types/studio";

const WORKSPACE_STORAGE_KEY = "rustc_playground_workspace_v1";

export function saveWorkspaceToLocalStorage(state: SavedWorkspaceState): void {
  try {
    window.localStorage.setItem(WORKSPACE_STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Ignore storage quota errors gracefully
  }
}

export function loadWorkspaceFromLocalStorage(): SavedWorkspaceState | null {
  try {
    const serialized = window.localStorage.getItem(WORKSPACE_STORAGE_KEY);
    if (!serialized) {
      return null;
    }
    return JSON.parse(serialized) as SavedWorkspaceState;
  } catch {
    return null;
  }
}

export function downloadTextFile(filename: string, content: string, mimeType = "text/plain"): void {
  const blob = new Blob([content], { type: mimeType });
  const objectUrl = URL.createObjectURL(blob);
  const anchorElement = document.createElement("a");
  anchorElement.href = objectUrl;
  anchorElement.download = filename;
  document.body.appendChild(anchorElement);
  anchorElement.click();
  document.body.removeChild(anchorElement);
  URL.revokeObjectURL(objectUrl);
}
