# password-manager

A desktop password generator and encrypted password manager for macOS, Windows, and Linux, built with Rust and Tauri 2 (web UI in TypeScript).

> **Status:** v0.1 in progress. The generator core (`crates/pm-core`) is built and tested; the desktop UI is next. Nothing is stored yet.

## v0.1: password generator

The rules engine lives in [`crates/pm-core`](crates/pm-core), a plain Rust library with no UI code:

- 15 rule options drawn from common website requirements, including an exact digit count (a 16th, no reuse across saved passwords, arrives with the vault)
- An optional memorable phrase, used as typed or transformed (random case and symbol swaps)
- Randomness only from the operating system's secure generator, with candidates that break a rule thrown away and rebuilt rather than patched
- A strength estimate that gives the phrase no credit
- Secrets wiped from memory when dropped (`zeroize`), and hidden from debug output
- No `unsafe` code

### Running the tests

Requires [Rust](https://rustup.rs) 1.90 or newer.

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`cargo test` runs the unit tests and property tests, `clippy` is the linter, and `fmt --check` confirms the formatting.

The property tests (`crates/pm-core/tests/properties.rs`) generate passwords for hundreds of random rule sets and check that every password follows every enabled rule.

## Planned features

- **Password generator:** pick rules from a list of common site requirements, set the exact number of digits, optionally build the password around a memorable phrase, and regenerate on demand.
- **Encrypted vault:** stores account name, username/email, and password. Encrypted with Argon2id and XChaCha20-Poly1305 (RustCrypto crates). Secrets stay in the Rust core; the UI never holds the vault key.
- **Multiple users:** separate profiles, each with its own vault and master password.
- **Cloud sync:** the encrypted vault file syncs through iCloud Drive, Dropbox, or Google Drive. The provider only sees ciphertext.
- **Biometric unlock:** Touch ID on macOS, Windows Hello on Windows.

## Documentation

- [Architecture and design](docs/architecture.md): components, security design, generator algorithm, sync and merge rules, and open questions.
- Architecture diagram: [PNG](docs/architecture.png) · [SVG](docs/architecture.svg) · [Mermaid source](docs/architecture.mmd)
- [Progress](progress.md): what each commit accomplishes, and what's next.
