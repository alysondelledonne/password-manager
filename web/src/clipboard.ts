// Copy with a best-effort automatic clear.
//
// After the delay, the clipboard is cleared only if it still holds the copied
// password, so something else the person copied in the meantime is never
// erased. Browsers that don't let the page read the clipboard can't make that
// check, so nothing is cleared and the person is told.
//
// The desktop app will do this from Rust instead, without these limits.

export const CLEAR_AFTER_MS = 30_000;

export type ClearResult = "cleared" | "changed" | "unreadable";

export interface ClipboardLike {
  writeText(text: string): Promise<void>;
  readText?(): Promise<string>;
}

/** Copies `text`, then reports what happened at clear time. Returns a cancel function. */
export async function copyWithAutoClear(
  text: string,
  clipboard: ClipboardLike,
  onClearAttempt: (result: ClearResult) => void,
  delayMs: number = CLEAR_AFTER_MS,
): Promise<() => void> {
  await clipboard.writeText(text);
  const timer = setTimeout(async () => {
    if (!clipboard.readText) {
      onClearAttempt("unreadable");
      return;
    }
    try {
      const current = await clipboard.readText();
      if (current === text) {
        await clipboard.writeText("");
        onClearAttempt("cleared");
      } else {
        onClearAttempt("changed");
      }
    } catch {
      onClearAttempt("unreadable");
    }
  }, delayMs);
  return () => clearTimeout(timer);
}
