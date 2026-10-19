//! EFF's short word list, bundled into the binary.
//!
//! Source: <https://www.eff.org/dice>, "EFF Short Wordlist 1".
//! License: CC BY 3.0 US, Electronic Frontier Foundation. See `data/README.md`.
//!
//! Each line is a dice number, a tab, and a word; only the word is used.
//! The file is bundled exactly as published (1,296 words). One word, `yo-yo`,
//! contains a hyphen; it is skipped when the list is loaded, because a hyphen is
//! a special character and would break the "no special characters" setting.

use std::sync::OnceLock;

const EFF_SHORT_WORDLIST: &str = include_str!("../../data/eff_short_wordlist_1.txt");

/// Every word in the file, in file order, including any that are skipped.
pub fn eff_short_all() -> impl Iterator<Item = &'static str> {
    EFF_SHORT_WORDLIST
        .lines()
        .filter_map(|line| line.split_whitespace().last())
}

/// The words the generator uses: those made only of lowercase letters a–z.
pub fn eff_short() -> &'static [&'static str] {
    static WORDS: OnceLock<Vec<&'static str>> = OnceLock::new();
    WORDS.get_or_init(|| eff_short_all().filter(|w| is_plain_word(w)).collect())
}

fn is_plain_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase())
}
