// Typed wrapper around the Rust generator compiled to WebAssembly
// (crates/pm-wasm). `npm run build:wasm` creates ./wasm/.

import { normalizeOutcome } from "./outcome";

export interface WordOptions {
  minLength: number;
  maxLength: number;
  /** Exactly this many digits in the whole password. */
  digits: number;
  uppercase: boolean;
  lowercase: boolean;
  /** At least one separator is one of `allowedSymbols`. Off: no special characters. */
  symbolsRequired: boolean;
  /** Single characters. */
  allowedSymbols: string[];
}

export type WordSource = { kind: "random" } | { kind: "mine"; words: string[] };

export type Field = "length" | "digits" | "letters" | "symbols" | "words";

export type Outcome =
  | {
      status: "ok";
      password: string;
      wordCount: number;
      entropyBits: number;
      /** "Hard to guess", 1 to 5 */
      guessStars: number;
      /** "Easy to remember", 1 to 5 */
      memoryStars: number;
    }
  | { status: "error"; code: string; field: Field | null; message: string };

export interface Generator {
  defaultOptions(): WordOptions;
  generate(options: WordOptions, source: WordSource): Outcome;
}

// Loading the module is the only part that needs the generated ./wasm/ files,
// so it's kept apart from the checks in ./outcome (testable without a build).
export async function loadGenerator(): Promise<Generator> {
  const wasm = await import("./wasm/pm_wasm.js");
  await wasm.default();
  return {
    defaultOptions: () => wasm.defaultOptions() as WordOptions,
    generate: (options, source) => normalizeOutcome(wasm.generate({ policy: options, words: source })),
  };
}
