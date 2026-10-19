# PassPhrase Pal

[![CI](https://github.com/alysondelledonne/password-manager/actions/workflows/ci.yml/badge.svg)](https://github.com/alysondelledonne/password-manager/actions/workflows/ci.yml)

**Try it: https://alysondelledonne.github.io/password-manager/**

Memorable, unique passwords, and later an encrypted password manager, built in Rust. v0.1 is a website: the Rust generator is compiled to WebAssembly and runs in your browser, so passwords never leave the page. The password manager will be a desktop app for macOS, Windows and Linux (Tauri 2).

> **Status:** v0.1 is live. Nothing is stored; passwords are made in your browser.

![PassPhrase Pal: Pal the elephant holding a word tile, next to a generated password and its two star ratings](docs/screenshot.png)

## What this demonstrates

- **Rust in the browser:** the generator is a Rust library compiled to WebAssembly (`wasm-bindgen`). Randomness comes from the browser's secure generator, and no password ever reaches a server.
- **Correctness by testing:** property tests (`proptest`) check every rule over thousands of random settings, and a word-list test caught a hyphenated word that would have slipped a special character into passwords set to have none. Sampling is exact: the engine counts the passwords that fit and picks uniformly, instead of guessing and retrying.
- **Security habits:** no `unsafe` code (forbidden by a workspace lint), secrets wiped from memory (`zeroize`) and hidden from debug output, a strict Content Security Policy, and honest ratings.
- **CI/CD:** GitHub Actions run formatting, lints and tests on macOS, Windows and Linux, then build and publish the site to GitHub Pages.

## v0.1: the PassPhrase Pal website

Makes passwords you can remember, like `Otter7Wagon!Maple2`: 2 or 3 words joined by digits and special characters.

- **Words:** enter a few of your own (2 or 3 are picked each time), or use random words from EFF's short word list.
- **Options:** length range (3 words if they fit, otherwise 2), exactly how many digits, uppercase and lowercase on or off, and whether special characters are required, picked from a set of bubbles. Spaces are never used.
- **Regenerate** gives a new password with the same settings. **Copy** clears the clipboard after 30 seconds, but only if it still holds the password.
- **Two star ratings**, because the goal is a different password for every site that you can remember without a password manager. **Easy to remember** counts how many separate things you'd have to recall. **Hard to guess** measures randomness. A one-line verdict weighs the two, and a guide below the generator explains both in plain language and says which accounts each level suits.
- **Design:** the password is shown in pieces (words, digits, special characters), each underlined and colored by kind, which is how the "Easy to remember" rating counts them. Type is Atkinson Hyperlegible, made by the Braille Institute so that look-alike characters such as `I l 1` and `O 0` stay distinct. Pal, the elephant mascot, is drawn in inline SVG, so the strict Content Security Policy needs no image sources.

How it fits together:

- [`crates/pm-core`](crates/pm-core): the rules, a plain Rust library with no UI code
- [`crates/pm-wasm`](crates/pm-wasm): compiles `pm-core` to WebAssembly for the page
- [`web/`](web): the page, in React + TypeScript

What's in `pm-core`:

- **`passphrase`** (used by the website): picks words, separators and digits uniformly among the passwords that fit the length range, and computes the strength exactly. It counts the fitting combinations rather than guessing and retrying.
- **`generator`** (random characters, kept for the desktop app): 15 rule options drawn from common website requirements, including an exact digit count. A 16th, no reuse across saved passwords, arrives with the vault.
- Randomness only from the operating system's secure generator; in the browser that's `crypto.getRandomValues`.
- Secrets wiped from memory when dropped (`zeroize`), and hidden from debug output.
- No `unsafe` code.

### Running the tests

Requires [Rust](https://rustup.rs) 1.90 or newer.

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`cargo test` runs the unit tests and property tests, `clippy` is the linter, and `fmt --check` confirms the formatting.

### Running the website locally

Also requires [Node.js](https://nodejs.org) 20 or newer. One-time setup:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cd web
npm install
```

The `wasm-bindgen-cli` version must match the `wasm-bindgen` version in `Cargo.lock`.

Then, from `web/`:

```sh
npm run dev
```

and open the address it prints (usually http://localhost:5173). `npm run dev` first builds the Rust generator to WebAssembly, then starts the page.

Page changes reload by themselves, but **Rust changes don't**: after changing anything under `crates/`, stop the server (Ctrl+C) and run `npm run dev` again. If you forget, the page says the generator module is out of date.

Other commands, from `web/`:

```sh
npm test
npm run typecheck
npm run build
npm run preview
```

`npm test` runs the page's tests, `typecheck` builds the WebAssembly bindings and checks the TypeScript, `build` creates the production site in `web/dist/` (with the Content Security Policy), and `preview` serves that build locally.

The property tests (`crates/pm-core/tests/properties.rs` and `passphrase_properties.rs`) generate passwords for hundreds of random settings and check that every password follows every rule. `tests/wordlist.rs` checks the bundled word list is the complete EFF list.

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

## Credits

Random words come from the [EFF Short Wordlist 1](https://www.eff.org/dice) by the Electronic Frontier Foundation, used under [CC BY 3.0 US](https://creativecommons.org/licenses/by/3.0/us/). See [`crates/pm-core/data/README.md`](crates/pm-core/data/README.md).

Fonts: [Atkinson Hyperlegible Next and Mono](https://www.brailleinstitute.org/freefont/) by the Braille Institute, under the [SIL Open Font License 1.1](https://openfontlicense.org), bundled through Fontsource.
