//! Regular verb conjugation.
//!
//! The query space is (person, number, tense, form); the rules only truly vary
//! along four output slots — 3rd-singular present, past finite, present
//! participle, past participle — which is also exactly the tuple shape the
//! generated verb table stores. `be` is fully suppletive and handled by a
//! dedicated table ([`EnglishCore::to_be`]) rather than the suffix rules; the
//! extractor deliberately skips `be` for the same reason.

use crate::EnglishCore;
use crate::grammar::*;
impl EnglishCore {
    /// Conjugate a verb by rule. The past participle intentionally reuses the
    /// past-finite rule (`-ed` serves both for every regular verb); genuinely
    /// distinct participles (`gone`, `eaten`) are irregular and live in the
    /// `english` crate's tables, not here.
    pub fn verb(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
    ) -> String {
        match word {
            "be" => {
                return EnglishCore::to_be(person, number, tense, form).to_string();
            }
            _ => (),
        }
        match (person, number, tense, form) {
            (_, _, _, Form::Infinitive) => {
                return word.to_string();
            }

            (Person::Third, Number::Singular, Tense::Present, Form::Finite) => {
                if let Some(irr) = EnglishCore::iter_replace_last(word, IRREGULAR_THIRD) {
                    return irr;
                }
                format!("{}{}", word, "s")
            }
            (_, _, Tense::Present, Form::Finite) => {
                return word.to_string();
            }
            (_, _, Tense::Present, Form::Participle) => {
                if let Some(irr) = EnglishCore::iter_replace_last(word, IRREGULAR_PRES_PART) {
                    return irr;
                }
                format!("{}{}", word, "ing")
            }

            (_, _, Tense::Past, _) => {
                if let Some(irr) = EnglishCore::iter_replace_last(word, IRREGULAR_PAST) {
                    return irr;
                }
                format!("{}{}", word, "ed")
            }
        }
    }
    /// The fully suppletive paradigm of `be` — the only English verb whose
    /// finite forms vary by person AND number, hence a lookup rather than rules.
    pub fn to_be(person: &Person, number: &Number, tense: &Tense, form: &Form) -> &'static str {
        match (tense, form) {
            (_, Form::Infinitive) => "be",
            (Tense::Present, Form::Finite) => match number {
                Number::Singular => match person {
                    Person::First => "am",
                    Person::Second => "are",
                    Person::Third => "is",
                },
                Number::Plural => "are",
            },
            (Tense::Past, Form::Finite) => match number {
                Number::Singular => match person {
                    Person::First => "was",
                    Person::Second => "were",
                    Person::Third => "was",
                },
                Number::Plural => "were",
            },
            (Tense::Past, Form::Participle) => "been",
            (Tense::Present, Form::Participle) => "being",
        }
    }
}

static IRREGULAR_PRES_PART: &[(&str, &str)] = &[
    ("e", "ing"),
    ("p", "pping"),
    ("ng", "nging"),
    ("g", "gging"),
    //  ("b", "bbing"),
    //   ("d", "dding"),
    //  ("t", "tting"),
];

static IRREGULAR_PAST: &[(&str, &str)] = &[
    ("fight", "fought"),
    ("buy", "bought"),
    ("e", "ed"),
    ("p", "pped"),
    ("y", "ied"),
    ("ng", "nged"),
    ("g", "gged"),
    // ("b", "bbed"),
    //("d", "dded"),
    //  ("t", "tted"),
];

static IRREGULAR_THIRD: &[(&str, &str)] = &[
    ("sh", "shes"),
    ("ch", "ches"),
    ("s", "ses"),
    ("z", "zes"),
    ("x", "xes"),
    ("buy", "buys"),
    ("y", "ies"),
];
