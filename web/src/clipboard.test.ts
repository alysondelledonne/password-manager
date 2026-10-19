import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { copyWithAutoClear, type ClipboardLike } from "./clipboard";

function fakeClipboard(readable = true) {
  let contents = "";
  const clipboard: ClipboardLike & { contents: () => string; set: (s: string) => void } = {
    writeText: vi.fn(async (t: string) => {
      contents = t;
    }),
    contents: () => contents,
    set: (s) => {
      contents = s;
    },
  };
  if (readable) clipboard.readText = vi.fn(async () => contents);
  return clipboard;
}

describe("copyWithAutoClear", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("copies, then clears after the delay if the clipboard still holds the password", async () => {
    const cb = fakeClipboard();
    const onClear = vi.fn();
    await copyWithAutoClear("Kp7!secret", cb, onClear, 30_000);
    expect(cb.contents()).toBe("Kp7!secret");

    await vi.advanceTimersByTimeAsync(29_999);
    expect(cb.contents()).toBe("Kp7!secret");

    await vi.advanceTimersByTimeAsync(1);
    expect(cb.contents()).toBe("");
    expect(onClear).toHaveBeenCalledWith("cleared");
  });

  it("never erases something else the person copied in the meantime", async () => {
    const cb = fakeClipboard();
    const onClear = vi.fn();
    await copyWithAutoClear("Kp7!secret", cb, onClear, 1000);
    cb.set("my shopping list");

    await vi.advanceTimersByTimeAsync(1000);
    expect(cb.contents()).toBe("my shopping list");
    expect(onClear).toHaveBeenCalledWith("changed");
  });

  it("leaves the clipboard alone when the browser won't let the page read it", async () => {
    const cb = fakeClipboard(true);
    cb.readText = vi.fn(async () => {
      throw new DOMException("denied", "NotAllowedError");
    });
    const onClear = vi.fn();
    await copyWithAutoClear("Kp7!secret", cb, onClear, 1000);

    await vi.advanceTimersByTimeAsync(1000);
    expect(cb.contents()).toBe("Kp7!secret");
    expect(onClear).toHaveBeenCalledWith("unreadable");
  });

  it("reports unreadable when the browser has no readText at all", async () => {
    const cb = fakeClipboard(false);
    const onClear = vi.fn();
    await copyWithAutoClear("Kp7!secret", cb, onClear, 1000);
    await vi.advanceTimersByTimeAsync(1000);
    expect(onClear).toHaveBeenCalledWith("unreadable");
  });

  it("can be cancelled before the delay", async () => {
    const cb = fakeClipboard();
    const onClear = vi.fn();
    const cancel = await copyWithAutoClear("Kp7!secret", cb, onClear, 1000);
    cancel();
    await vi.advanceTimersByTimeAsync(5000);
    expect(onClear).not.toHaveBeenCalled();
    expect(cb.contents()).toBe("Kp7!secret");
  });
});
