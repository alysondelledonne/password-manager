// Checks what the WebAssembly module returned. Kept separate from generator.ts
// so it can be tested before the module has been built.

import type { Field, Outcome } from "./generator";

type RawOutcome =
  | Extract<Outcome, { status: "ok" }>
  | { status: "error"; code: string; field?: Field | null; message: string };

/** Message shown when the compiled Rust module doesn't match this page. */
export const STALE_MODULE_MESSAGE =
  "The generator module is out of date with this page. Stop the dev server (Ctrl+C) and run npm run dev again to rebuild it.";

/**
 * Checks what the WebAssembly module returned. A module built from older Rust
 * code (the dev server only rebuilds it on start) can be missing fields; say so
 * instead of showing an empty rating.
 */
export function normalizeOutcome(raw: unknown): Outcome {
  const r = raw as Partial<RawOutcome> | null;
  if (r?.status === "error") {
    return { status: "error", code: r.code ?? "unknown", field: r.field ?? null, message: r.message ?? "" };
  }
  const ok = r as Partial<Extract<Outcome, { status: "ok" }>> | null;
  const isStars = (n: unknown) => typeof n === "number" && n >= 1 && n <= 5;
  if (
    ok?.status === "ok" &&
    typeof ok.password === "string" &&
    typeof ok.wordCount === "number" &&
    typeof ok.entropyBits === "number" &&
    isStars(ok.guessStars) &&
    isStars(ok.memoryStars)
  ) {
    return ok as Outcome;
  }
  return { status: "error", code: "stale_module", field: null, message: STALE_MODULE_MESSAGE };
}
