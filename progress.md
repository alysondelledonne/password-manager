# Progress

What each commit to this repo accomplishes, newest first. The plan behind these steps is in [docs/architecture.md](docs/architecture.md) (Section 11, build phases).

**How to keep this up to date:** add the entry for a change in the same commit as the change, using the commit message as the heading. Hashes aren't known until after committing, so fill in the hash when the next entry is written.

## Current status

| Stage | Status |
|---|---|
| Design (architecture doc + diagram) | ✅ Done, pushed |
| v0.1 step 1: generator core (`pm-core`) with tests | 🟡 Done locally, not yet committed |
| v0.1 step 2: Tauri app shell | ⏳ Next |
| v0.1 step 3: React generator screen | ⏳ Planned |
| v0.1 step 4: CI on macOS/Windows/Linux + README screenshot | ⏳ Planned |
| Later stages: vault, encryption, profiles, sync, biometrics | ⏳ Planned (architecture doc, Section 11) |

---

## Commit log

### Add pm-core password generator with tests
- **Commit:** not yet committed · **Pushed:** no
- **Suggested commit message:** `Add pm-core password generator with unit and property tests`

**What it accomplishes:** the first working code. A Rust library, `crates/pm-core`, that generates passwords from a rule set. No UI and no storage yet.

- **All 16 rule options** from the architecture doc (R1–R16, Section 4.1): length limits, required uppercase, lowercase and symbols, an **exact** digit count, an allowed-symbol set, no spaces, must start with a letter, look-alike exclusion, repeat limit, no sequences (`abc`, `321`, `qwe`), no username, not a common password, ASCII only. R16 (reuse across vault entries) waits for the vault.
- **Optional memorable phrase** in two modes: as-is, or transformed (random case, plus `a→@`, `s→$`, `i→!` when allowed). A phrase that breaks a rule is rejected with a message saying what to change.
- **Secure randomness:** characters come only from the OS random number generator. The code only accepts a cryptographically secure generator (`CryptoRng`), so a weak one can't be passed in by mistake.
- **Rejection sampling:** a candidate that breaks a rule is thrown away and rebuilt, never patched, so every valid password is equally likely.
- **Strength estimate** in bits, with a rating (Weak / Fair / Strong / Very strong). The phrase gets zero credit.
- **Memory hygiene:** passwords and phrases are wiped from memory when dropped (`zeroize`) and print as `[REDACTED]` in debug output.
- **No `unsafe` code**, enforced by a workspace lint.

**Also in this commit:**
- `docs/architecture.md`: records the decisions from 2026-10-19 (React frontend; symbol-only swaps; phrases with digits rejected; rule-breaking phrases rejected rather than edited; v0.1 scope). Adds open questions 17 (should the repeat, sequence and look-alike rules apply to the phrase?) and 18 (license).
- `README.md`: v0.1 section, and test commands that work when pasted into zsh.
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

## Planned commits for v0.1

These are the next steps, so each commit stays small enough to review in one sitting.

1. **Tauri app shell:** `src-tauri/` with one IPC command, `generate`, that calls `pm-core`. Strict Content Security Policy, and permissions that let the UI call only that command. Nothing is saved to disk.
2. **React generator screen:** controls for every rule, the digit count, the phrase and its mode, and a Generate / Regenerate button. Shows the password with its strength rating and a Copy button. Phrase errors appear next to the phrase field.
3. **CI + screenshot:** GitHub Actions running `cargo test`, `clippy` and `fmt` on macOS, Windows and Linux, plus a screenshot in the README.
