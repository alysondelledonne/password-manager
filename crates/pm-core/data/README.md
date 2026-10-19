# Data files

## `eff_short_wordlist_1.txt`

"EFF Short Wordlist 1": 1,296 short, everyday words for generating memorable passwords. Each line is a dice number, a tab, and a word.

- Source: https://www.eff.org/dice (file: https://www.eff.org/files/2016/09/08/eff_short_wordlist_1.txt)
- Author: Electronic Frontier Foundation
- License: Creative Commons Attribution 3.0 United States (CC BY 3.0 US), https://creativecommons.org/licenses/by/3.0/us/
- Changes: none. The file is bundled exactly as published; `tests/wordlist.rs` checks it has all 1,296 unique words.
- Use: one word, `yo-yo`, is skipped when the list is loaded (`src/passphrase/wordlist.rs`), because its hyphen is a special character and would break the "no special characters" setting. The generator uses the other 1,295.

## `common-passwords.txt`

A short placeholder list of widely reported most-common passwords, checked against every generated password. To be replaced with a larger maintained list before release.
