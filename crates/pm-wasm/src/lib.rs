//! WebAssembly bindings for the website (v0.1): memorable word-based passwords.
//!
//! The page sends the word options and where the words come from (random
//! words, or the person's own). All rules live in `pm-core`'s `passphrase`
//! module; this crate only converts to and from JavaScript.
//!
//! Randomness: `pm-core` uses the OS random number generator. In the browser
//! that is `crypto.getRandomValues`, through getrandom's `wasm_js` backend.

use pm_core::passphrase::{PassphrasePolicy, PassphraseRequest, generate_passphrase_with_os_rng};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// What the page gets back. Never an exception: errors are values the UI shows.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Outcome {
    Ok {
        password: String,
        word_count: usize,
        entropy_bits: f64,
        /// "Hard to guess", 1 to 5.
        guess_stars: u8,
        /// "Easy to remember", 1 to 5.
        memory_stars: u8,
    },
    Error {
        /// Stable machine-readable code, e.g. `word_has_digit`.
        code: String,
        /// Which part of the form to highlight: `length`, `digits`, `letters`,
        /// `symbols`, `words`, or none.
        field: Option<String>,
        /// Plain-English message to show the person.
        message: String,
    },
}

/// Generates a password. Plain Rust, so it can be tested without a browser.
pub fn generate_words(request: &PassphraseRequest) -> Outcome {
    match generate_passphrase_with_os_rng(request) {
        Ok(out) => Outcome::Ok {
            password: out.password.to_string(),
            word_count: out.word_count,
            entropy_bits: out.entropy_bits,
            guess_stars: out.guess_stars,
            memory_stars: out.memory_stars,
        },
        Err(err) => Outcome::Error {
            code: err.code().to_string(),
            field: err.field().map(str::to_string),
            message: err.to_string(),
        },
    }
}

/// JavaScript: `defaultOptions()` → the starting values for the form.
#[wasm_bindgen(js_name = defaultOptions)]
pub fn default_options() -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&PassphrasePolicy::default()).map_err(Into::into)
}

/// JavaScript: `generate({ policy, words })`, where `words` is
/// `{ kind: "random" }` or `{ kind: "mine", words: ["maple", "otter"] }`.
/// Returns `{ status: "ok", ... }` or `{ status: "error", code, field, message }`.
#[wasm_bindgen]
pub fn generate(request: JsValue) -> Result<JsValue, JsValue> {
    let outcome = match serde_wasm_bindgen::from_value::<PassphraseRequest>(request) {
        Ok(req) => generate_words(&req),
        Err(err) => Outcome::Error {
            code: "invalid_input".to_string(),
            field: None,
            message: format!("The page sent options the generator couldn't read: {err}"),
        },
    };
    serde_wasm_bindgen::to_value(&outcome).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(json: &str) -> PassphraseRequest {
        serde_json::from_str(json).unwrap()
    }

    const POLICY: &str = r##"{"minLength":12,"maxLength":30,"digits":2,"uppercase":true,"lowercase":true,
        "symbolsRequired":true,"allowedSymbols":["!","#"]}"##;

    #[test]
    fn reads_random_words_request_from_the_page_shape() {
        let req = request(&format!(
            r#"{{"policy":{POLICY},"words":{{"kind":"random"}}}}"#
        ));
        match generate_words(&req) {
            Outcome::Ok {
                password,
                word_count,
                ..
            } => {
                assert!(word_count == 2 || word_count == 3);
                assert_eq!(password.chars().filter(char::is_ascii_digit).count(), 2);
            }
            other => panic!("expected a password, got {other:?}"),
        }
    }

    #[test]
    fn reads_my_words_request_from_the_page_shape() {
        let req = request(&format!(
            r#"{{"policy":{POLICY},"words":{{"kind":"mine","words":["maple","otter","lantern"]}}}}"#
        ));
        match generate_words(&req) {
            Outcome::Ok { password, .. } => {
                let lower = password.to_lowercase();
                let used = ["maple", "otter", "lantern"]
                    .iter()
                    .filter(|w| lower.contains(*w))
                    .count();
                assert!(used >= 2, "{password}");
            }
            other => panic!("expected a password, got {other:?}"),
        }
    }

    #[test]
    fn word_errors_point_at_the_words_field() {
        let req = request(&format!(
            r#"{{"policy":{POLICY},"words":{{"kind":"mine","words":["summer2019","maple"]}}}}"#
        ));
        match generate_words(&req) {
            Outcome::Error {
                code,
                field,
                message,
            } => {
                assert_eq!(code, "word_has_digit");
                assert_eq!(field.as_deref(), Some("words"));
                assert!(message.contains("summer2019"));
            }
            other => panic!("expected an error, got {other:?}"),
        }
    }

    #[test]
    fn outcome_json_shape_matches_what_the_page_expects() {
        let ok = Outcome::Ok {
            password: "x".into(),
            word_count: 3,
            entropy_bits: 1.5,
            guess_stars: 1,
            memory_stars: 4,
        };
        let json = serde_json::to_string(&ok).unwrap();
        for part in [
            r#""status":"ok""#,
            r#""wordCount":3"#,
            r#""entropyBits":1.5"#,
            r#""guessStars":1"#,
            r#""memoryStars":4"#,
        ] {
            assert!(json.contains(part), "{json} should contain {part}");
        }
    }

    #[test]
    fn default_options_use_the_page_field_names() {
        let json = serde_json::to_string(&PassphrasePolicy::default()).unwrap();
        for part in [
            "minLength",
            "maxLength",
            "digits",
            "symbolsRequired",
            "allowedSymbols",
        ] {
            assert!(json.contains(part), "{json} should contain {part}");
        }
    }
}
