# password-manager

A desktop password generator and encrypted password manager for macOS, Windows, and Linux, built with Rust and Tauri 2 (web UI in TypeScript).

> **Status:** Design phase. No application code yet. Not ready to store real passwords.

## Planned features

- **Password generator:** pick rules from a list of common site requirements, set the exact number of digits, optionally build the password around a memorable phrase, and regenerate on demand.
- **Encrypted vault:** stores account name, username/email, and password. Encrypted with Argon2id and XChaCha20-Poly1305 (RustCrypto crates). Secrets stay in the Rust core; the UI never holds the vault key.
- **Multiple users:** separate profiles, each with its own vault and master password.
- **Cloud sync:** the encrypted vault file syncs through iCloud Drive, Dropbox, or Google Drive. The provider only sees ciphertext.
- **Biometric unlock:** Touch ID on macOS, Windows Hello on Windows.

## Documentation

- [Architecture and design](docs/architecture.md): components, security design, generator algorithm, sync and merge rules, and open questions.
- Architecture diagram: [PNG](docs/architecture.png) · [SVG](docs/architecture.svg) · [Mermaid source](docs/architecture.mmd)
