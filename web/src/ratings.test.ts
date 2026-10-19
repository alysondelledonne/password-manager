import { describe, expect, it } from "vitest";
import { GUESS_RATINGS, MEMORY_RATINGS, verdict } from "./ratings";

describe("ratings", () => {
  it("has one entry per star level, highest first", () => {
    expect(GUESS_RATINGS.map((r) => r.stars)).toEqual([5, 4, 3, 2, 1]);
    expect(MEMORY_RATINGS.map((r) => r.stars)).toEqual([5, 4, 3, 2, 1]);
  });

  it("calls a password hard to remember first, whatever its strength", () => {
    expect(verdict(2, 5)).toMatch(/^Hard to remember\./);
    expect(verdict(1, 1)).toMatch(/^Hard to remember\./);
  });

  it("calls it a good balance when both are at least 3 stars", () => {
    expect(verdict(3, 3)).toBe("Good balance: fine to memorize for everyday accounts.");
    expect(verdict(5, 4)).toBe("Good balance: fine to memorize for accounts with money or private details.");
  });

  it("says when it's easy to remember but weak, and how to fix it", () => {
    expect(verdict(5, 1)).toBe(
      "Easy to remember, but only strong enough for throwaway sign-ups. Random words, or more of your own words, make it harder to guess.",
    );
  });
});
