import { describe, expect, it } from "vitest";
import { STALE_MODULE_MESSAGE, normalizeOutcome } from "./generator";

const ok = { status: "ok", password: "Maple4Otter!Wagon7", wordCount: 3, entropyBits: 42.4, guessStars: 3, memoryStars: 3 };

describe("normalizeOutcome", () => {
  it("passes a complete result through", () => {
    expect(normalizeOutcome(ok)).toEqual(ok);
  });

  it("reports an out-of-date module instead of showing empty stars", () => {
    // What a module built before the two ratings returned.
    const old = { status: "ok", password: "Maple4Otter!Wagon7", wordCount: 3, entropyBits: 42.4, strength: "weak" };
    expect(normalizeOutcome(old)).toEqual({
      status: "error",
      code: "stale_module",
      field: null,
      message: STALE_MODULE_MESSAGE,
    });
    expect(normalizeOutcome({ ...ok, guessStars: 0 })).toMatchObject({ code: "stale_module" });
  });

  it("fills in a missing field on errors (Rust's None arrives as undefined)", () => {
    expect(normalizeOutcome({ status: "error", code: "too_strict", message: "m" })).toEqual({
      status: "error",
      code: "too_strict",
      field: null,
      message: "m",
    });
  });
});
