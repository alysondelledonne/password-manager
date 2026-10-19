//! Checks the bundled EFF short word list is the complete, unmodified list,
//! and that the generator uses only plain-letter words from it.

use std::collections::BTreeSet;

use pm_core::passphrase::wordlist::{eff_short, eff_short_all};

#[test]
fn file_has_all_1296_unique_words() {
    let all: Vec<&str> = eff_short_all().collect();
    assert_eq!(
        all.len(),
        1296,
        "EFF Short Wordlist 1 has 6^4 = 1,296 words"
    );
    let unique: BTreeSet<_> = all.iter().collect();
    assert_eq!(unique.len(), all.len(), "no duplicates");
}

#[test]
fn only_yo_yo_is_skipped() {
    let skipped: Vec<&str> = eff_short_all()
        .filter(|w| !eff_short().contains(w))
        .collect();
    assert_eq!(skipped, ["yo-yo"], "the one word with a hyphen");
    assert_eq!(eff_short().len(), 1295);
}

#[test]
fn used_words_are_lowercase_letters_only() {
    for w in eff_short() {
        assert!(
            !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase()),
            "unexpected word {w:?}"
        );
    }
}
