import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Generator, Outcome, WordOptions, WordSource } from "./generator";

// The real generator is Rust compiled to WebAssembly and is tested in Rust.
// These tests check the page: what it sends, and what it shows for each result.
const DEFAULTS: WordOptions = {
  minLength: 12,
  maxLength: 24,
  digits: 2,
  uppercase: true,
  lowercase: true,
  symbolsRequired: true,
  allowedSymbols: ["!", "#", "$", "%", "&", "*", "+", "-", "=", "?", "@", "^", "_"],
};

let nextOutcome: (options: WordOptions, source: WordSource) => Outcome;
const generate = vi.fn((options: WordOptions, source: WordSource) => nextOutcome(options, source));
const fakeGenerator: Generator = { defaultOptions: () => structuredClone(DEFAULTS), generate };

vi.mock("./generator", () => ({ loadGenerator: () => Promise.resolve(fakeGenerator) }));

const { App } = await import("./App");

let counter = 0;
const ok = (): Outcome => ({
  status: "ok",
  password: `Maple7Otter!Wagon${++counter}`,
  wordCount: 3,
  entropyBits: 42.4,
  guessStars: 3,
  memoryStars: 3,
});

// Mirrors the Rust rule: fewer than 2 of your own words is the starting prompt.
const realistic = (_o: WordOptions, source: WordSource): Outcome =>
  source.kind === "mine" && source.words.length < 2
    ? { status: "error", code: "too_few_words", field: "words", message: "Enter at least 2 different words, or switch to random words." }
    : ok();

const lastCall = () => generate.mock.calls.at(-1)!;

beforeEach(() => {
  counter = 0;
  generate.mockClear();
  nextOutcome = realistic;
});

async function addWords(user: ReturnType<typeof userEvent.setup>, words: string[]) {
  const input = screen.getByLabelText("Add a word you'll remember");
  for (const w of words) await user.type(input, `${w}{Enter}`);
}

describe("memorable password page", () => {
  it("starts by asking for words, without showing it as an error", async () => {
    render(<App />);
    const readout = await screen.findByRole("region", { name: "Generated password" });
    expect(within(readout).getByText("Add your words")).toBeInTheDocument();
    expect(within(readout).queryByRole("alert")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Copy" })).toBeDisabled();
    expect(lastCall()).toEqual([DEFAULTS, { kind: "mine", words: [] }]);
  });

  it("adds words with Enter and generates once there are two", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Add your words");

    await addWords(user, ["maple", "otter"]);

    expect(lastCall()).toEqual([DEFAULTS, { kind: "mine", words: ["maple", "otter"] }]);
    expect(screen.getByTestId("password")).toHaveTextContent(/^Maple7Otter!Wagon\d+$/);
    expect(within(screen.getByRole("list", { name: "Your words" })).getAllByRole("listitem")).toHaveLength(2);
  });

  it("removes a word", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Add your words");
    await addWords(user, ["maple", "otter", "wagon"]);

    await user.click(screen.getByRole("button", { name: "Remove otter" }));

    expect(lastCall()[1]).toEqual({ kind: "mine", words: ["maple", "wagon"] });
  });

  it("switches to random words", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Add your words");

    await user.click(screen.getByRole("radio", { name: "Random words" }));

    expect(lastCall()[1]).toEqual({ kind: "random" });
    expect(screen.getByTestId("password")).toBeInTheDocument();
    expect(screen.getByText(/3 words, about 42 bits/)).toBeInTheDocument();
  });

  it("regenerates on request with the same settings", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("radio", { name: "Random words" }));
    const first = screen.getByTestId("password").textContent;

    await user.click(screen.getByRole("button", { name: "Regenerate" }));

    expect(lastCall()).toEqual([DEFAULTS, { kind: "random" }]);
    expect(screen.getByTestId("password").textContent).not.toBe(first);
  });

  it("sends the digit count, letter switches and length range", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("radio", { name: "Random words" }));

    await user.click(screen.getByRole("button", { name: "More: Digits, exactly" }));
    await user.click(screen.getByRole("switch", { name: "Lowercase letters" }));
    const longest = screen.getByLabelText("Longest");
    await user.clear(longest);
    await user.type(longest, "30");

    expect(lastCall()[0]).toEqual({ ...DEFAULTS, digits: 3, lowercase: false, maxLength: 30 });
  });

  it("selects special characters with bubbles, and hides them when not required", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("radio", { name: "Random words" }));

    const hash = screen.getByRole("button", { name: "Allow #" });
    expect(hash).toHaveAttribute("aria-pressed", "true");
    await user.click(hash);
    expect(lastCall()[0].allowedSymbols).not.toContain("#");
    await user.click(screen.getByRole("button", { name: "Allow ~" }));
    expect(lastCall()[0].allowedSymbols).toContain("~");

    await user.click(screen.getByRole("switch", { name: "Special characters required" }));
    expect(lastCall()[0].symbolsRequired).toBe(false);
    expect(screen.queryByRole("group", { name: "Allowed special characters" })).not.toBeInTheDocument();
  });

  it("shows a word error under the words and in the readout", async () => {
    nextOutcome = (_o, source) =>
      source.kind === "mine" && source.words.includes("summer2019")
        ? {
            status: "error",
            code: "word_has_digit",
            field: "words",
            message: '"summer2019" contains a number. Remove it; digits are added separately.',
          }
        : realistic(_o, source);
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Add your words");

    await addWords(user, ["summer2019", "maple"]);

    const words = screen.getByRole("group", { name: "Words" });
    expect(within(words).getByText(/contains a number/)).toBeInTheDocument();
    expect(words).toHaveAttribute("aria-invalid", "true");
    const readout = screen.getByRole("region", { name: "Generated password" });
    expect(within(readout).getByRole("alert")).toHaveTextContent("contains a number");
    expect(screen.getByRole("button", { name: "Regenerate" })).toBeDisabled();
  });

  it("copies the password and says when the clipboard will be cleared", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("radio", { name: "Random words" }));
    const password = screen.getByTestId("password").textContent;

    await user.click(screen.getByRole("button", { name: "Copy" }));

    expect(await navigator.clipboard.readText()).toBe(password);
    expect(screen.getByText(/In 30 seconds the clipboard will be cleared/)).toBeInTheDocument();
  });

  it("shows both ratings and a verdict", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("radio", { name: "Random words" }));

    const readout = screen.getByRole("region", { name: "Generated password" });
    expect(within(readout).getByRole("img", { name: "Easy to remember: 3 out of 5 stars" })).toBeInTheDocument();
    expect(within(readout).getByRole("img", { name: "Hard to guess: 3 out of 5 stars" })).toBeInTheDocument();
    expect(within(readout).getByText(/Good balance: fine to memorize for everyday accounts\./)).toBeInTheDocument();
  });

  it("explains both ratings below and marks the current ones", async () => {
    nextOutcome = (o, source) => ({
      ...(realistic(o, source) as Extract<Outcome, { status: "ok" }>),
      memoryStars: 5,
      guessStars: 1,
    });
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole("radio", { name: "Random words" }));

    const guide = screen.getByRole("region", { name: "What the ratings mean" });
    const memory = within(guide).getByRole("list", { name: "Easy to remember ratings" });
    const guess = within(guide).getByRole("list", { name: "Hard to guess ratings" });
    const memoryRows = within(memory).getAllByRole("listitem");
    const guessRows = within(guess).getAllByRole("listitem");
    expect(memoryRows).toHaveLength(5);
    expect(guessRows).toHaveLength(5);
    expect(memoryRows[0]).toHaveAttribute("aria-current", "true");
    expect(memoryRows[0]).toHaveTextContent("Sticks after using it once or twice.");
    expect(guessRows[4]).toHaveAttribute("aria-current", "true");
    expect(guessRows[4]).toHaveTextContent("Throwaway sign-ups: newsletters, free trials, guest Wi-Fi.");
    expect(guide.querySelectorAll('[aria-current="true"]')).toHaveLength(2);
    expect(within(guide).getByText(/Make a new password for every site/)).toBeInTheDocument();

    const readout = screen.getByRole("region", { name: "Generated password" });
    expect(within(readout).getByText(/Easy to remember, but only strong enough for throwaway sign-ups/)).toBeInTheDocument();
  });

  it("marks no rating while there's no password", async () => {
    render(<App />);
    await screen.findByText("Add your words");
    const guide = screen.getByRole("region", { name: "What the ratings mean" });
    expect(guide.querySelectorAll('[aria-current="true"]')).toHaveLength(0);
  });

  it("stops accepting words at 10", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Add your words");
    await addWords(user, ["aa", "bb", "cc", "dd", "ee", "ff", "gg", "hh", "ii", "jj"]);
    expect(screen.getByLabelText("Add a word you'll remember")).toBeDisabled();
  });
});

describe("page shell", () => {
  it("is titled PassPhrase Pal", async () => {
    render(<App />);
    expect(await screen.findByRole("heading", { level: 1, name: "PassPhrase Pal" })).toBeInTheDocument();
  });

  it("splits a password into the pieces you'd remember, without changing it", async () => {
    const { chunks } = await import("./App");
    expect(chunks("Maple7Otter!Wagon42")).toEqual(["Maple", "7", "Otter", "!", "Wagon", "42"]);
    expect(chunks("maple!!otter").join("")).toBe("maple!!otter");
    expect(chunks("")).toEqual([]);
  });
});
