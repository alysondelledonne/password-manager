# Progress

What each commit to this repo accomplishes, newest first. The plan behind these steps is in [docs/architecture.md](docs/architecture.md) (Section 11, build phases).

**How to keep this up to date:** add the entry for a change in the same commit as the change, using the commit message as the heading. Hashes aren't known until after committing, so fill in the hash when the next entry is written.

## Current status

| Stage | Status |
|---|---|
| Design (architecture doc + diagram) | ✅ Done, pushed |
| v0.1 step 1: generator core (`pm-core`) with tests | ✅ Done, pushed (`afe8f08`) |
| v0.1 step 2: memorable password website (`passphrase` + `pm-wasm` + React page) | ✅ Done, pushed (`073c567`, `78c2a1c`) |
| v0.1 step 3: CI on macOS/Windows/Linux, GitHub Pages, README screenshot | ✅ CI and Pages pushed (`edf77f5`); 🟡 screenshot not yet committed |
| Desktop app (Tauri), then vault, encryption, profiles, sync, biometrics | ⏳ Planned (architecture doc, Section 11) |

On 2026-10-19 v0.1 changed from a desktop app to a website, and then from random-character to memorable word-based passwords (architecture doc, Sections 1, 3.4 and 4.6).

---

## Commit log

### Add README screenshot and page title
- **Commit:** not yet committed · **Pushed:** no
- **Suggested commit message:** `Add README screenshot and page title`

**What it accomplishes:** finishes v0.1.
- **`docs/screenshot.png`** (new): the live site, shown at the top of the README.
- **`web/index.html`:** the browser tab title and page description now say "Memorable password generator", matching the page heading. The old tab title, "Password generator", showed up when checking the live site.

---

### edf77f5 · Add CI and GitHub Pages deployment
- **Commit:** `edf77f5` · 2026-10-19 · **Pushed:** yes, merged in pull request #3. The site was live after the merge.

**What it accomplishes:** every push and pull request is tested automatically, and the website is published at https://alysondelledonne.github.io/password-manager/. A pull request missing files, like #1, now shows a red X before it's merged.

- **`.github/workflows/ci.yml`** (new): on every push to `main` and every pull request.
  - **Rust**, on macOS, Windows and Linux: `cargo fmt --check`, `clippy -D warnings`, `cargo test`.
  - **Website:** page tests, then a full production build: Rust to WebAssembly, type check, and Vite.
  - The `wasm-bindgen` CLI is pinned to 0.2.129 to match `Cargo.lock`.
- **`.github/workflows/pages.yml`** (new): on every push to `main`, runs the Rust and page tests, builds the site, and publishes it to GitHub Pages. A failing test stops the publish. Needs Settings → Pages → Source: "GitHub Actions" (done once, by hand).
- **`README.md`:** CI badge, live link, and a "What this demonstrates" section. The screenshot comes in a follow-up commit.

**How it was verified:** both workflows pass `actionlint`, with action versions checked against their repositories. The website job's steps were run in a clean copy of the repo, with fresh `npm ci` and no generated files; all 25 page tests pass without the WebAssembly module. The first real run is on GitHub after this is pushed.

---

### 073c567 + 78c2a1c · Memorable password generator website (Rust to WebAssembly + React)
- **`073c567`** · 2026-10-19 · **Pushed:** yes, merged in pull request #1. The page itself (`web/` only).
- **`78c2a1c`** · 2026-10-19 · **Pushed:** yes, merged in pull request #2. Everything else below: the Rust word generator, the WebAssembly layer, the word list, the out-of-date-module fix (`web/src/outcome.ts`) and the docs. It reuses the first commit's message. Between the two merges, `main` couldn't build the website.

**What it accomplishes:** the first version you can use. A web page that makes memorable passwords like `Otter7Wagon!Maple2` in the browser, using Rust compiled to WebAssembly. Nothing is sent to a server and nothing is stored.

- **`crates/pm-core/src/passphrase/`** (new):
  - **Words:** 2 or 3 of them, from the person's own (2–10 entered, picked at random) or from EFF's short word list. 3 if any 3-word password fits the length range, otherwise 2.
  - **Separators:** special characters or digits. Digits used as separators count toward the exact digit total, and the rest go at the end. When special characters are required, at least one separator is a selected symbol.
  - **Capitals:** the first letter of each word, or all caps or all lowercase when only one case is on.
  - **Spaces:** never. Words with spaces, digits or non-letters are rejected with a message, never edited.
  - **Picking:** counts the combinations that fit and picks uniformly among them, instead of guessing and retrying.
  - **Strength:** exact, with the person's own words counting as zero.
- **`crates/pm-core/data/eff_short_wordlist_1.txt`**: EFF Short Wordlist 1 (1,296 words, CC BY 3.0 US), bundled unchanged and credited in `data/README.md`, the README and the page footer. One word, `yo-yo`, is skipped when loading because its hyphen is a special character; 1,295 are used.
- **Two star ratings** (`passphrase/rating.rs`), because the purpose is a unique password per site that can be remembered without a password manager:
  - **Easy to remember:** counts "chunks" to recall. Own word 0.75, random word 1, each separator 1, trailing digits 1 per pair. Up to 4.5 chunks is 5 stars, up to 5.5 is 4, up to 6.5 is 3, up to 8 is 2, and more is 1.
  - **Hard to guess:** from exact bits. Under 20 is 1 star, 20–35 is 2, 35–45 is 3, 45–60 is 4, and 60 or more is 5.
  - **Own words:** now get credit for the random choice and order of words, still treating the words themselves as known to an attacker.
  - **Examples:** defaults are 3 and 3 stars; 2 own words are 5 to remember and 1 to guess; random words with 4 digits are 2 and 4.
- **`crates/pm-wasm`** (new): passes the page's word options to `pm-core` and returns a password with its word count and strength, or an error with a message and the field to highlight. Randomness is the browser's `crypto.getRandomValues`.
- **`web/`** (new): Vite + React + TypeScript page in dark grey, lime green, light grey and mustard yellow.
  - **Words:** "My words" (add with Enter, remove with ×) or "Random words".
  - **Options:** shortest and longest length, digits stepper, uppercase and lowercase switches, special characters switch with symbol bubbles.
  - **Regenerate**, and **Copy**, which clears the clipboard after 30 seconds only if it still holds the password.
  - **Password colors:** words in light grey, digits in mustard, special characters in lime.
  - **Starting state:** the page opens by asking for words, not with an error.
  - **Ratings:** the readout shows "Easy to remember" and "Hard to guess" stars and a verdict, e.g. "Good balance: fine to memorize for everyday accounts." Below the generator, "What the ratings mean" explains both scales side by side and highlights the current ones. It adds "One password per site" advice (no reuse, no per-site variations, two-step verification) and tips for balancing the two.
- **Out-of-date module check:** if the compiled Rust module is older than the page (the dev server only rebuilds it on start), the page says so and how to fix it, instead of showing empty stars. Found when the stars didn't fill in on the Mac.
- **Security settings:** strict Content Security Policy in the built site (no external requests, no inline scripts or styles, no `eval`; `'wasm-unsafe-eval'` only to run WebAssembly). Fonts are bundled.
- **Docs:** architecture doc Sections 3.4 (the website) and 4.6 (memorable passwords, including an honest note on strength), plus the decisions table. README and this file updated.
- The random-character generator (`generator` module) is unchanged and still tested, kept for the desktop app.

**How it was verified:**
- Rust: unit tests for every rule above. Property tests run 1,024 random settings with both random and own words: every password has the exact digit count, a length in range, at least one selected symbol when required (none when not), 2–3 words from the right source with the right capitals, no repeated own words, and no spaces; every error is explained by the settings. Further tests check that random sampling is uniform, that the combination counting is exact, that both rating scales follow their tables and match each password's actual shape (property-tested), that real-list ratings are right (defaults 3 and 3 stars; 4 digits 2 to remember and 4 to guess), and that the bundled list has all 1,296 unique words with only `yo-yo` skipped. `clippy` and `fmt` are clean.
- Page: 25 Vitest tests: 3 for the out-of-date module check, 13 for the page (adding and removing words, switching to random words, what each control sends, symbol bubbles, errors under the right field, copy, the 10-word limit, both ratings and the verdict, the guide's highlights), 4 for the verdict wording, and 5 for copy-and-clear. TypeScript type-checks.
- Browser: the production build was loaded in Chromium at desktop and phone sizes: no console errors, no policy violations, no requests to other sites. Clicking through it exposed a bug in the number boxes, which were corrected on every keystroke (clearing "Longest" and typing 30 gave 128). They now accept typing and are tidied when you leave the field.
- Real WebAssembly: the module built on the developer's Mac (first version of the page) generated 1,000 correct, all-different passwords when run directly, which confirms the browser random-number setup. The cloud workspace can't build WebAssembly itself, so its browser screenshots used a stand-in module; the redesigned page was confirmed running on the Mac.
- Real word list: checking the downloaded list found `yo-yo`, whose hyphen would have slipped a special character into passwords with special characters turned off. It's now skipped, with a test.

---

### 450f016 · Fix property test generating impossible rule sets
- **Commit:** `450f016` · 2026-10-19 · **Pushed:** yes (`origin/main`)

**What it accomplishes:** fixes a test bug, not a generator bug. The random rule sets in `crates/pm-core/tests/properties.rs` could require 13 characters (3 uppercase, 3 lowercase, 4 digits, 3 symbols) in a 12-character password. The generator correctly rejected that with `RequiredExceedsLength`, but the test expected every rule set to be valid, so it failed now and then. The test now builds only rule sets that fit (`prop_filter`). Verified with 50 consecutive runs (about 25,000 random rule sets), plus `clippy` and `fmt`.

---

### 1b4778f · Fix README rule count and test commands; update progress log
- **Commit:** `1b4778f` · 2026-10-19 · **Pushed:** yes (`origin/main`)

**What it accomplishes:** corrects documentation pushed in `afe8f08`. No code changes.

- `README.md`: says 15 rule options, not 16 (R16 needs the vault). Removes the comment after `cargo test`, which broke the command when pasted into zsh, the macOS default shell. Adds a sentence explaining each test command and a link to this file.
- `progress.md`: fills in the `afe8f08` hash, corrects that entry's rule count, and adds this entry.

---

### afe8f08 · Add pm-core password generator with unit and property tests
- **Commit:** `afe8f08` · 2026-10-19 · **Pushed:** yes (`origin/main`)

**What it accomplishes:** the first working code. A Rust library, `crates/pm-core`, that generates passwords from a rule set. No UI and no storage yet.

- **15 of the 16 rule options** from the architecture doc (R1–R15, Section 4.1): length limits, required uppercase, lowercase and symbols, an **exact** digit count, an allowed-symbol set, no spaces, must start with a letter, look-alike exclusion, repeat limit, no sequences (`abc`, `321`, `qwe`), no username, not a common password, ASCII only. R16 (reuse across vault entries) waits for the vault.
- **Optional memorable phrase** in two modes: as-is, or transformed (random case, plus `a→@`, `s→$`, `i→!` when allowed). A phrase that breaks a rule is rejected with a message saying what to change.
- **Secure randomness:** characters come only from the OS random number generator. The code only accepts a cryptographically secure generator (`CryptoRng`), so a weak one can't be passed in by mistake.
- **Rejection sampling:** a candidate that breaks a rule is thrown away and rebuilt, never patched, so every valid password is equally likely.
- **Strength estimate** in bits, with a rating (Weak / Fair / Strong / Very strong). The phrase gets zero credit.
- **Memory hygiene:** passwords and phrases are wiped from memory when dropped (`zeroize`) and print as `[REDACTED]` in debug output.
- **No `unsafe` code**, enforced by a workspace lint.

**Also in this commit:**
- `docs/architecture.md`: records the decisions from 2026-10-19 (React frontend; symbol-only swaps; phrases with digits rejected; rule-breaking phrases rejected rather than edited; v0.1 scope). Adds open questions 17 (should the repeat, sequence and look-alike rules apply to the phrase?) and 18 (license).
- `README.md`: v0.1 section with the test commands. (As pushed, it said 16 rule options and had a comment after `cargo test` that breaks in zsh; both fixed in the next commit.)
- `progress.md`: this file.

**Key files:**
- `Cargo.toml`: workspace settings and lints
- `Cargo.lock`: exact dependency versions, committed because this is an application
- `crates/pm-core/src/generator/engine.rs`: the algorithm (start here)
- `crates/pm-core/src/generator/`: also `policy.rs`, `phrase.rs`, `rules.rs`, `charsets.rs`, `strength.rs` and `error.rs`
- `crates/pm-core/tests/properties.rs`: property tests
- `crates/pm-core/data/common-passwords.txt`: placeholder blocklist

**How it was verified:**
- `cargo test`: 36 tests pass. That's 33 unit tests and 3 integration tests, including 2 property tests that run 512 random rule sets each.
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are both clean.
- `cargo test --release` passes too, so nothing depends on debug-only checks.
- A deliberately planted bug (digits allowed into the fill pool) was caught at once by the property tests (`exact digits`), then removed.

**Known limitations:**
- The common-password list is a ~50-entry placeholder.
- No license yet (open question 18).

---

### e5a69cf · initial files
- **Commit:** `e5a69cf` · 2026-10-19 · **Pushed:** yes (`origin/main`)

**What it accomplishes:** the project's design, before any code.

- `docs/architecture.md`, design draft v2: Rust + Tauri 2 desktop app for macOS, Windows and Linux. Covers the generator algorithm, the vault's key hierarchy and file format, the webview security boundary, per-entry sync merging, biometrics, testing, build phases, risks and open questions. Draft v1 (Python) was replaced before this commit; Section 1.1 explains why.
- `docs/architecture.png`, `.svg` and `.mmd`: the architecture diagram, plus its editable Mermaid source.
- `README.md`: project summary and links to the docs.
- `.gitignore`: keeps vault files, `profiles.json`, keys and `.env` files out of Git, along with Rust, Tauri and Node build output.

---

## Planned commits

These are the next steps, so each commit stays small enough to review in one sitting.

1. **CI + hosting (finishes v0.1):** GitHub Actions running the Rust and page tests on macOS, Windows and Linux; build the site and publish it to GitHub Pages; README screenshot and live link.
2. **Desktop app shell:** Tauri app reusing `pm-core` and the React page, with copy and clear done in Rust.
3. **Vault:** encryption and storage (architecture doc, Section 5).
