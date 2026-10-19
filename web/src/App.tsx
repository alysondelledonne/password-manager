import { useEffect, useId, useMemo, useRef, useState, type CSSProperties, type FormEvent } from "react";
import { loadGenerator, type Field, type Generator, type WordOptions, type WordSource } from "./generator";
import { GUESS_RATINGS, MEMORY_RATINGS, verdict } from "./ratings";
import { CLEAR_AFTER_MS, copyWithAutoClear, type ClearResult } from "./clipboard";
import { BrandMark, Elephant, GuessIcon, MemoryIcon } from "./Art";

const MAX_LENGTH = 128;
const REPO_URL = "https://github.com/alysondelledonne/password-manager";
/** Faded tiles drifting behind Pal in the banner: the kinds of pieces a password is made of. */
const DRIFT_TILES = ["maple", "7", "harbor", "!", "quilt", "42", "ember", "#"];
const MAX_WORDS = 10;

/** The special characters offered as bubbles. Defaults come from Rust. */
export const SYMBOL_CHOICES = ["!", "@", "#", "$", "%", "^", "&", "*", "-", "_", "=", "+", "?", ".", ":", "~"];

const CLEAR_NOTES: Record<ClearResult, string> = {
  cleared: "Clipboard cleared.",
  changed: "Clipboard left as is: it no longer holds this password.",
  unreadable:
    "This browser didn't let the page check the clipboard, so it wasn't cleared. Copy something else to replace it.",
};

type LoadState =
  | { kind: "loading" }
  | { kind: "failed"; message: string }
  | { kind: "ready"; generator: Generator };

export function App() {
  const [load, setLoad] = useState<LoadState>({ kind: "loading" });

  useEffect(() => {
    let active = true;
    loadGenerator()
      .then((generator) => active && setLoad({ kind: "ready", generator }))
      .catch(
        (err: unknown) =>
          active && setLoad({ kind: "failed", message: err instanceof Error ? err.message : String(err) }),
      );
    return () => {
      active = false;
    };
  }, []);

  return (
    <div className="page">
      <a className="skip-link" href="#generator">
        Skip to the generator
      </a>
      <header className="topbar">
        <a className="brand" href="#top" aria-label="PassPhrase Pal, top of page">
          <BrandMark size={28} />
          <span>PassPhrase Pal</span>
        </a>
        <nav aria-label="On this page">
          <a href="#ratings">Ratings</a>
          <a href="#how-it-works">How it works</a>
          <a href={REPO_URL}>Source</a>
        </nav>
      </header>

      <section className="hero" id="top" aria-labelledby="hero-title">
        <div className="hero-copy">
          <h1 id="hero-title" className="wordmark">
            <span>Pass</span>
            <span>Phrase</span> <span>Pal</span>
          </h1>
          <p className="tagline">Passwords that stick in your brain, not on a sticky note.</p>
          <p className="lede">
            Two or three real words with a pinch of digits and special characters, made in your
            browser. Nothing you type or generate leaves your device.
          </p>
        </div>
        <div className="hero-art">
          <ul className="drift" aria-hidden="true">
            {DRIFT_TILES.map((t) => (
              <li key={t} className={`drift-${charClass(t[0])}`}>
                {t}
              </li>
            ))}
          </ul>
          <Elephant word="otter" />
        </div>
      </section>

      <div id="generator" className="generator" tabIndex={-1}>
        {load.kind === "loading" && <p className="status">Starting the generator…</p>}
        {load.kind === "failed" && (
          <p className="status status-error" role="alert">
            The generator couldn't start. It needs WebAssembly, which current versions of Chrome,
            Firefox, Safari and Edge all support. Details: {load.message}
          </p>
        )}
        {load.kind === "ready" && <GeneratorPanel generator={load.generator} />}
      </div>

      <HowItWorks />

      <footer className="colophon">
        <div className="colophon-row">
          <span className="colophon-brand">
            <BrandMark size={22} />
            PassPhrase Pal v0.1
          </span>
          <a href={REPO_URL}>Source on GitHub</a>
          <span>
            Random words from <a href="https://www.eff.org/dice">EFF&apos;s short word list</a> (CC BY
            3.0)
          </span>
        </div>
      </footer>
    </div>
  );
}

function GeneratorPanel({ generator }: { generator: Generator }) {
  const [options, setOptions] = useState<WordOptions>(() => generator.defaultOptions());
  const [mode, setMode] = useState<WordSource["kind"]>("mine");
  const [myWords, setMyWords] = useState<string[]>([]);
  const [regenerations, setRegenerations] = useState(0);
  const [copyNote, setCopyNote] = useState<string | null>(null);
  const cancelClear = useRef<(() => void) | null>(null);

  const source: WordSource = mode === "random" ? { kind: "random" } : { kind: "mine", words: myWords };

  // Regenerates whenever an option changes or Regenerate is pressed.
  const outcome = useMemo(
    () => generator.generate(options, source),
    // `regenerations` is a dependency on purpose: bumping it asks for a fresh password.
    [generator, options, mode, myWords, regenerations],
  );

  useEffect(() => setCopyNote(null), [outcome]);
  useEffect(() => () => cancelClear.current?.(), []);

  const set =
    <K extends keyof WordOptions>(key: K) =>
    (value: WordOptions[K]) =>
      setOptions((current) => ({ ...current, [key]: value }));

  const errorFor = (field: Field) =>
    outcome.status === "error" && outcome.field === field ? outcome.message : null;
  // Fewer than 2 words isn't a mistake, it's the starting point: show it as a prompt.
  const needsWords = outcome.status === "error" && outcome.code === "too_few_words";

  async function copy() {
    if (outcome.status !== "ok") return;
    try {
      cancelClear.current?.();
      cancelClear.current = await copyWithAutoClear(outcome.password, navigator.clipboard, (result) =>
        setCopyNote(CLEAR_NOTES[result]),
      );
      setCopyNote(
        `Copied. In ${CLEAR_AFTER_MS / 1000} seconds the clipboard will be cleared if it still holds this password.`,
      );
    } catch {
      setCopyNote("Couldn't copy. Select the password and copy it yourself.");
    }
  }

  function toggleSymbol(symbol: string) {
    setOptions((current) => {
      const selected = current.allowedSymbols.includes(symbol)
        ? current.allowedSymbols.filter((s) => s !== symbol)
        : [...current.allowedSymbols, symbol];
      return { ...current, allowedSymbols: SYMBOL_CHOICES.filter((s) => selected.includes(s)) };
    });
  }

  return (
    <>
      <main className="bench">
        <section className="readout" aria-label="Generated password">
          {outcome.status === "ok" ? (
            <>
              <output className="password" aria-live="polite" data-testid="password">
                {/* Keyed so a new password replays the entrance; the live region itself stays put. */}
                <span key={`${outcome.password}-${regenerations}`} className="chunks">
                  {chunks(outcome.password).map((chunk, i) => (
                    <span key={i} className={`chunk c-${charClass(chunk)}`} style={{ "--i": i } as CSSProperties}>
                      {chunk}
                    </span>
                  ))}
                </span>
              </output>
              <ul className="legend" aria-label="Colors in the password">
                <li className="legend-letter">Words</li>
                <li className="legend-digit">Digits</li>
                <li className="legend-symbol">Special characters</li>
              </ul>
              <RatingSummary
              memoryStars={outcome.memoryStars}
              guessStars={outcome.guessStars}
              bits={outcome.entropyBits}
              words={outcome.wordCount}
            />
            </>
          ) : (
            <p className="password password-empty">{needsWords ? "Add your words" : "No password yet"}</p>
          )}
          {outcome.status === "error" && (
            <p className={needsWords ? "readout-prompt" : "readout-error"} role={needsWords ? undefined : "alert"}>
              {needsWords
                ? "Enter at least 2 words you'll remember, or switch to random words."
                : outcome.message}
            </p>
          )}
          <div className="actions">
            <button
              type="button"
              className="primary"
              onClick={() => setRegenerations((n) => n + 1)}
              disabled={outcome.status !== "ok"}
            >
              Regenerate
            </button>
            <button type="button" onClick={copy} disabled={outcome.status !== "ok"}>
              Copy
            </button>
          </div>
          <p className="copy-note" aria-live="polite">
            {copyNote}
          </p>
        </section>

        <form className="controls" onSubmit={(e) => e.preventDefault()} noValidate>
          <fieldset className="group" aria-invalid={errorFor("words") && !needsWords ? true : undefined}>
            <legend>Words</legend>
            <div className="segmented" role="radiogroup" aria-label="Where the words come from">
              <SegmentButton active={mode === "mine"} onClick={() => setMode("mine")}>
                My words
              </SegmentButton>
              <SegmentButton active={mode === "random"} onClick={() => setMode("random")}>
                Random words
              </SegmentButton>
            </div>
            {mode === "mine" ? (
              <MyWords words={myWords} onChange={setMyWords} />
            ) : (
              <p className="hint">
                2 or 3 words picked at random from EFF's list of short, everyday words.
              </p>
            )}
            {!needsWords && errorFor("words") && <FieldError message={errorFor("words")!} />}
          </fieldset>

          <fieldset className="group" aria-invalid={errorFor("length") ? true : undefined}>
            <legend>Length</legend>
            <div className="row-2">
              <NumberField label="Shortest" value={options.minLength} min={1} onChange={set("minLength")} />
              <NumberField label="Longest" value={options.maxLength} min={1} onChange={set("maxLength")} />
            </div>
            <p className="hint">Uses 3 words when they fit in this range, otherwise 2.</p>
            {errorFor("length") && <FieldError message={errorFor("length")!} />}
          </fieldset>

          <fieldset
            className="group"
            aria-invalid={errorFor("digits") || errorFor("letters") || errorFor("symbols") ? true : undefined}
          >
            <legend>Characters</legend>
            <Stepper label="Digits, exactly" value={options.digits} onChange={set("digits")} />
            {errorFor("digits") && <FieldError message={errorFor("digits")!} />}
            <Switch label="Uppercase letters" checked={options.uppercase} onChange={set("uppercase")} />
            <Switch label="Lowercase letters" checked={options.lowercase} onChange={set("lowercase")} />
            {errorFor("letters") && <FieldError message={errorFor("letters")!} />}
            <Switch
              label="Special characters required"
              checked={options.symbolsRequired}
              onChange={set("symbolsRequired")}
            />
            {options.symbolsRequired && (
              <div className="chips" role="group" aria-label="Allowed special characters">
                {SYMBOL_CHOICES.map((s) => (
                  <button
                    key={s}
                    type="button"
                    className="chip"
                    aria-pressed={options.allowedSymbols.includes(s)}
                    aria-label={`Allow ${s}`}
                    onClick={() => toggleSymbol(s)}
                  >
                    {s}
                  </button>
                ))}
              </div>
            )}
            {errorFor("symbols") && <FieldError message={errorFor("symbols")!} />}
          </fieldset>
        </form>
      </main>
      <RatingGuide current={outcome.status === "ok" ? outcome : null} />
    </>
  );
}

/** "letter", "digit" or "symbol", from the first character. */
function charClass(text: string): "letter" | "digit" | "symbol" {
  if (/^[0-9]/.test(text)) return "digit";
  if (/^\p{L}/u.test(text)) return "letter";
  return "symbol";
}

/** Splits a password into the pieces you'd remember: words, runs of digits, and special characters. */
export function chunks(password: string): string[] {
  return password.match(/\p{L}+|[0-9]+|[^\p{L}0-9]/gu) ?? [];
}

function MyWords({ words, onChange }: { words: string[]; onChange: (words: string[]) => void }) {
  const [draft, setDraft] = useState("");
  const id = useId();
  const full = words.length >= MAX_WORDS;

  function add(e: FormEvent) {
    e.preventDefault();
    const word = draft.trim();
    if (!word || full) return;
    onChange([...words, word]);
    setDraft("");
  }

  return (
    <div className="my-words">
      <label htmlFor={id}>Add a word you'll remember</label>
      <div className="add-row">
        <input
          id={id}
          type="text"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && add(e)}
          spellCheck={false}
          autoComplete="off"
          aria-describedby={`${id}-help`}
          disabled={full}
        />
        <button type="button" onClick={add} disabled={full || !draft.trim()}>
          Add
        </button>
      </div>
      <p className="hint" id={`${id}-help`}>
        Add 2 to {MAX_WORDS} words; 2 or 3 are picked each time. People who know you could guess
        them, so they add no strength: that comes from the digits and special characters.
      </p>
      {words.length > 0 && (
        <ul className="word-chips" aria-label="Your words">
          {words.map((w, i) => (
            <li key={`${w}-${i}`}>
              <span>{w}</span>
              <button
                type="button"
                aria-label={`Remove ${w}`}
                onClick={() => onChange(words.filter((_, j) => j !== i))}
              >
                ×
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function SegmentButton(props: { active: boolean; onClick: () => void; children: string }) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={props.active}
      className={props.active ? "segment active" : "segment"}
      onClick={props.onClick}
    >
      {props.children}
    </button>
  );
}

function Stars({ count, label }: { count: number; label: string }) {
  return (
    <span className="stars" role="img" aria-label={`${label}: ${count} out of 5 stars`}>
      {[1, 2, 3, 4, 5].map((n) => (
        <svg key={n} viewBox="0 0 24 24" aria-hidden="true" className={n <= count ? "star on" : "star"}>
          <path d="M12 2.5l2.9 6.1 6.6.8-4.9 4.6 1.3 6.6L12 17.3l-5.9 3.3 1.3-6.6-4.9-4.6 6.6-.8z" />
        </svg>
      ))}
    </span>
  );
}

function RatingSummary(props: { memoryStars: number; guessStars: number; bits: number; words: number }) {
  return (
    <div className="rating-summary">
      <dl className="rating-rows">
        <div>
          <dt>Easy to remember</dt>
          <dd>
            <Stars count={props.memoryStars} label="Easy to remember" />
          </dd>
        </div>
        <div>
          <dt>Hard to guess</dt>
          <dd>
            <Stars count={props.guessStars} label="Hard to guess" />
          </dd>
        </div>
      </dl>
      <p className="verdict">
        {verdict(props.memoryStars, props.guessStars)} <a href="#ratings">What the ratings mean</a>
      </p>
      <p className="bits">
        {props.words} words, about {Math.round(props.bits)} bits of randomness.
      </p>
    </div>
  );
}

function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

function RatingGuide({ current }: { current: { memoryStars: number; guessStars: number } | null }) {
  return (
    <section className="guide" id="ratings" aria-labelledby="ratings-title">
      <h2 id="ratings-title">What the ratings mean</h2>
      <p className="guide-intro">
        The goal is a different password for every site that you can still remember without a
        password manager. Being easy to remember and hard to guess pull against each other, so
        each password gets both ratings. For everyday accounts, aim for at least 3 stars on both.
      </p>

      <div className="guide-columns">
        <div>
          <h3 className="guide-heading">
            <MemoryIcon />
            Easy to remember
          </h3>
          <p className="guide-note">
            How many separate things you have to recall: words, digits and special characters.
          </p>
          <ul className="guide-list" aria-label="Easy to remember ratings">
            {MEMORY_RATINGS.map((r) => (
              <GuideRow
                key={r.stars}
                stars={r.stars}
                label="Easy to remember"
                title={r.title}
                text={r.description}
                current={current?.memoryStars === r.stars}
              />
            ))}
          </ul>
        </div>
        <div>
          <h3 className="guide-heading">
            <GuessIcon />
            Hard to guess
          </h3>
          <p className="guide-note">
            Which accounts it&apos;s fine to memorize for. The risk isn&apos;t someone typing guesses
            at a login page, but a site being breached and attackers guessing offline, very fast.
          </p>
          <ul className="guide-list" aria-label="Hard to guess ratings">
            {GUESS_RATINGS.map((r) => (
              <GuideRow
                key={r.stars}
                stars={r.stars}
                label="Hard to guess"
                title={r.title}
                text={`${capitalize(r.goodFor)}: ${r.examples}`}
                current={current?.guessStars === r.stars}
              />
            ))}
          </ul>
        </div>
      </div>

      <div className="guide-tips">
        <div>
          <h3>One password per site</h3>
          <ul>
            <li>
              Make a new password for every site, even when they&apos;re all memorable. When a site
              is breached, attackers try the leaked password on other sites first.
            </li>
            <li>
              Don&apos;t reuse one password with a small change per site, such as adding the
              site&apos;s name. One leak gives the pattern away.
            </li>
            <li>
              Turn on two-step verification for your email and bank. It protects you even if a
              password is guessed.
            </li>
          </ul>
        </div>
        <div>
          <h3>Finding the balance</h3>
          <ul>
            <li>
              Your own words are the easiest to remember. Enter more of them, up to 10: the more
              there are to pick from, the harder the password is to guess.
            </li>
            <li>
              Each extra digit or special character makes it harder to guess, and harder to
              remember.
            </li>
            <li>
              Regenerate until you get one that sticks. Picking from a handful costs very little
              strength.
            </li>
          </ul>
        </div>
      </div>
    </section>
  );
}

function HowItWorks() {
  return (
    <section className="how" id="how-it-works" aria-labelledby="how-title">
      <h2 id="how-title">How it works</h2>
      <p className="section-intro">
        PassPhrase Pal has no server to send anything to. The page is a static file, and the
        password is made on your device.
      </p>
      <div className="how-grid">
        <div>
          <h3>Rust, in your browser</h3>
          <p>
            The generator is written in Rust and compiled to WebAssembly, so the same tested code
            will run in the desktop app planned next.
          </p>
        </div>
        <div>
          <h3>Secure randomness</h3>
          <p>
            Words, digits and special characters are picked with your browser&apos;s
            cryptographic random number generator, the one used for encryption keys.
          </p>
        </div>
        <div>
          <h3>Nothing kept</h3>
          <p>
            No accounts, cookies or tracking. Reload the page and your words are gone. A copied
            password is cleared from the clipboard after {CLEAR_AFTER_MS / 1000} seconds, unless
            you've copied something else since.
          </p>
        </div>
      </div>
    </section>
  );
}

function GuideRow(props: { stars: number; label: string; title: string; text: string; current: boolean }) {
  return (
    <li className={props.current ? "current" : undefined} aria-current={props.current ? "true" : undefined}>
      <Stars count={props.stars} label={props.label} />
      <div>
        <h4>
          {props.title}
          {props.current && <span className="you-are-here">Your password</span>}
        </h4>
        <p>{props.text}</p>
      </div>
    </li>
  );
}

function FieldError({ message }: { message: string }) {
  return <p className="field-error">{message}</p>;
}

function clamp(n: number, min: number): number {
  if (!Number.isFinite(n)) return min;
  return Math.max(min, Math.min(MAX_LENGTH, Math.trunc(n)));
}

/** A number box you can clear and retype: the value is applied as you type and tidied when you leave it. */
function NumberInput(props: {
  value: number;
  min: number;
  onChange: (n: number) => void;
  label?: string;
  id?: string;
}) {
  const [draft, setDraft] = useState(String(props.value));
  const [editing, setEditing] = useState(false);
  useEffect(() => {
    if (!editing) setDraft(String(props.value));
  }, [props.value, editing]);
  return (
    <input
      id={props.id}
      type="number"
      inputMode="numeric"
      aria-label={props.label}
      min={props.min}
      max={MAX_LENGTH}
      value={draft}
      onFocus={() => setEditing(true)}
      onChange={(e) => {
        setDraft(e.target.value);
        if (e.target.value.trim() !== "") props.onChange(clamp(Number(e.target.value), props.min));
      }}
      onBlur={() => {
        setEditing(false);
        setDraft(String(props.value));
      }}
    />
  );
}

function NumberField(props: { label: string; value: number; min: number; onChange: (n: number) => void }) {
  const id = useId();
  return (
    <div className="number-field">
      <label htmlFor={id}>{props.label}</label>
      <NumberInput id={id} value={props.value} min={props.min} onChange={props.onChange} />
    </div>
  );
}

function Stepper(props: { label: string; value: number; onChange: (n: number) => void }) {
  const { label, value, onChange } = props;
  return (
    <div className="stepper" role="group" aria-label={label}>
      <span className="stepper-label">{label}</span>
      <button
        type="button"
        aria-label={`Fewer: ${label}`}
        onClick={() => onChange(clamp(value - 1, 0))}
        disabled={value <= 0}
      >
        −
      </button>
      <NumberInput label={label} value={value} min={0} onChange={onChange} />
      <button type="button" aria-label={`More: ${label}`} onClick={() => onChange(clamp(value + 1, 0))}>
        +
      </button>
    </div>
  );
}

function Switch(props: { label: string; checked: boolean; onChange: (checked: boolean) => void }) {
  const id = useId();
  return (
    <div className="switch-row">
      <label htmlFor={id}>{props.label}</label>
      <input
        id={id}
        type="checkbox"
        role="switch"
        className="switch"
        checked={props.checked}
        onChange={(e) => props.onChange(e.target.checked)}
      />
    </div>
  );
}
