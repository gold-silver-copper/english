use english_core::EnglishCore;
pub use english_core::grammar::*;

mod noun_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/noun_phf.rs"
    ));
}
use noun_phf::*;
mod adj_phf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/adj_phf.rs"));
}
use adj_phf::*;
mod verb_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/verb_phf.rs"
    ));
}
use verb_phf::*;
#[cfg(feature = "senses")]
mod variants_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/variants_phf.rs"
    ));
}
#[cfg(feature = "senses")]
use variants_phf::*;
#[cfg(feature = "dictionary")]
mod dictionary_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/dictionary_phf.rs"
    ));
}

/// Strips the sense-disambiguation suffix from a key. Assignment suffixes are
/// allocated append-only and may grow past a single digit, so we strip *all*
/// trailing ASCII digits (lemmas themselves never contain digits).
fn strip_trailing_number(word: &str) -> &str {
    word.trim_end_matches(|c: char| c.is_ascii_digit())
}

/// Entry point for English inflection and morphology.
///
/// `English` is the low-level `&str` API for handling English nouns, verbs,
/// adjectives, pronouns, and possessives. It delegates irregular forms to
/// lookup tables and falls back on [`EnglishCore`] for regular inflection
/// rules. For noun counting ergonomics, see [`count`] and
/// [`count_with_number`].
pub struct English;
impl English {
    /// Inflects a noun into singular or plural form.
    ///
    /// Handles irregular nouns (e.g., `"child" -> "children"`) and
    /// falls back to regular pluralization rules when no override is found.
    /// Strips trailing numbers used for sense disambiguation (`"die2" -> "dice"`).
    ///
    /// # Examples
    /// ```rust
    /// use english::{English, Number};
    ///
    /// assert_eq!(English::noun("cat", &Number::Plural), "cats");
    /// assert_eq!(English::noun("child", &Number::Plural), "children");
    /// assert_eq!(English::noun("die2", &Number::Plural), "dice");
    /// ```
    pub fn noun(word: &str, number: &Number) -> String {
        let base_word = strip_trailing_number(word);

        match number {
            Number::Singular => base_word.to_string(),
            Number::Plural => {
                if let Some(x) = get_plural(word) {
                    x.to_owned()
                } else {
                    EnglishCore::noun(base_word, number)
                }
            }
        }
    }

    /// Inflects an adjective into positive, comparative, or superlative form.
    ///
    /// Handles irregular adjectives (e.g., `"good" -> "better"/"best"`)
    /// and falls back to regular periphrastic forms
    /// (e.g., `"fun" -> "more fun"/"most fun"`).
    /// Strips trailing numbers used for disambiguation (`"bad3"` -> `"worse"`).
    ///
    /// # Examples
    /// ```rust
    /// use english::{Degree, English};
    ///
    /// assert_eq!(English::adj("fast", &Degree::Comparative), "more fast");
    /// assert_eq!(English::adj("good2", &Degree::Superlative), "best");
    /// assert_eq!(English::adj("fun", &Degree::Comparative), "more fun");
    /// ```
    pub fn adj(word: &str, degree: &Degree) -> String {
        let base_word = strip_trailing_number(word);
        match degree {
            Degree::Positive => base_word.to_owned(),
            Degree::Comparative => {
                if let Some((comp, _)) = get_adjective_forms(word) {
                    comp.to_owned()
                } else {
                    EnglishCore::comparative(base_word)
                }
            }
            Degree::Superlative => {
                if let Some((_, sup)) = get_adjective_forms(word) {
                    sup.to_owned()
                } else {
                    EnglishCore::superlative(base_word)
                }
            }
        }
    }

    /// Conjugates a verb into the requested form.
    ///
    /// Handles irregular verbs (e.g., `"go" -> "went"`, `"eat" -> "ate"`)
    /// and falls back to regular conjugation rules when no override is found.
    /// Strips trailing numbers used for sense disambiguation (`"lie2"` -> `"lied"`).
    ///
    /// # Examples
    /// ```rust
    /// use english::{English, Form, Number, Person, Tense};
    ///
    /// // Regular verb
    /// assert_eq!(
    ///     English::verb("walk", &Person::Third, &Number::Singular, &Tense::Present, &Form::Finite),
    ///     "walks"
    /// );
    ///
    /// // Irregular verb
    /// assert_eq!(
    ///     English::verb("eat", &Person::Third, &Number::Singular, &Tense::Past, &Form::Finite),
    ///     "ate"
    /// );
    ///
    /// // Participle
    /// assert_eq!(
    ///     English::verb("go", &Person::Third, &Number::Plural, &Tense::Past, &Form::Participle),
    ///     "gone"
    /// );
    /// ```
    pub fn verb(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
    ) -> String {
        let base_word = strip_trailing_number(word);
        match get_verb_forms(word) {
            Some(wordik) => match (person, number, tense, form) {
                (_, _, _, Form::Infinitive) => base_word.to_owned(),
                (Person::Third, Number::Singular, Tense::Present, Form::Finite) => {
                    wordik.0.to_string()
                }
                (_, _, Tense::Present, Form::Finite) => base_word.to_owned(),
                (_, _, Tense::Present, Form::Participle) => wordik.2.to_owned(),
                (_, _, Tense::Past, Form::Participle) => wordik.3.to_owned(),
                (_, _, Tense::Past, Form::Finite) => wordik.1.to_owned(),
            },
            None => EnglishCore::verb(base_word, person, number, tense, form),
        }
    }
    /// Returns the correct English pronoun for the given grammatical features.
    ///
    /// # Examples
    /// ```rust
    /// use english::{Case, English, Gender, Number, Person};
    ///
    /// assert_eq!(
    ///     English::pronoun(&Person::First, &Number::Singular, &Gender::Neuter, &Case::Nominative),
    ///     "I"
    /// );
    /// assert_eq!(
    ///     English::pronoun(&Person::Third, &Number::Singular, &Gender::Feminine, &Case::Nominative),
    ///     "she"
    /// );
    /// assert_eq!(
    ///     English::pronoun(&Person::Third, &Number::Plural, &Gender::Neuter, &Case::Nominative),
    ///     "they"
    /// );
    /// ```
    pub fn pronoun(person: &Person, number: &Number, gender: &Gender, case: &Case) -> &'static str {
        EnglishCore::pronoun(person, number, gender, case)
    }
    /// Adds an English possessive suffix (`'s` or `'`) to a word.
    ///
    /// # Examples
    /// ```rust
    /// use english::English;
    ///
    /// assert_eq!(English::add_possessive("dog"), "dog's");
    /// assert_eq!(English::add_possessive("dogs"), "dogs'");
    /// ```
    pub fn add_possessive(word: &str) -> String {
        EnglishCore::add_possessive(word)
    }

    /// Lists the explicit disambiguation keys this crate stores for a noun lemma.
    ///
    /// Returns every key sharing the base lemma (e.g. `["die2"]`, or `["lie",
    /// "lie2"]` for verbs) when the lemma is polysemous, and an empty slice when
    /// it has a single sense. This lets callers discover which numbered variants
    /// exist instead of hard-coding suffixes that could shift between releases —
    /// the keys themselves are pinned by the assignment lockfiles and stable.
    ///
    /// # Examples
    /// ```rust
    /// use english::English;
    ///
    /// assert_eq!(English::noun_senses("die"), &["die2"]);
    /// assert!(English::noun_senses("cat").is_empty());
    /// ```
    #[cfg(feature = "senses")]
    pub fn noun_senses(lemma: &str) -> &'static [&'static str] {
        noun_variants(strip_trailing_number(lemma)).unwrap_or(&[])
    }

    /// Lists the explicit disambiguation keys this crate stores for a verb lemma.
    ///
    /// # Examples
    /// ```rust
    /// use english::English;
    ///
    /// assert_eq!(English::verb_senses("lie"), &["lie", "lie2"]);
    /// ```
    #[cfg(feature = "senses")]
    pub fn verb_senses(lemma: &str) -> &'static [&'static str] {
        verb_variants(strip_trailing_number(lemma)).unwrap_or(&[])
    }

    /// Lists the explicit disambiguation keys this crate stores for an adjective lemma.
    ///
    /// # Examples
    /// ```rust
    /// use english::English;
    ///
    /// assert_eq!(English::adj_senses("bad"), &["bad2", "bad3"]);
    /// ```
    #[cfg(feature = "senses")]
    pub fn adj_senses(lemma: &str) -> &'static [&'static str] {
        adj_variants(strip_trailing_number(lemma)).unwrap_or(&[])
    }

    /// Returns the Wiktionary definitions for a noun key.
    ///
    /// Keyed by the exact, sense-disambiguated key (e.g. `"die2"`), so distinct
    /// homographs return distinct definitions. Irregular/homograph keys carry their
    /// full sense list; common regular words carry a single primary definition.
    /// Returns an empty slice for unknown or uncommon words. Requires the
    /// `dictionary` feature.
    ///
    /// # Examples
    /// ```rust
    /// # #[cfg(feature = "dictionary")] {
    /// use english::English;
    /// assert!(English::noun_meanings("die2")[0].to_lowercase().contains("cube"));
    /// assert!(!English::noun_meanings("cat").is_empty()); // common regular word
    /// # }
    /// ```
    #[cfg(feature = "dictionary")]
    pub fn noun_meanings(key: &str) -> &'static [&'static str] {
        dictionary_phf::noun_meanings(key).unwrap_or(&[])
    }

    /// Returns the Wiktionary definitions for a verb **sense key** (e.g. `"lie2"`).
    /// Requires the `dictionary` feature.
    #[cfg(feature = "dictionary")]
    pub fn verb_meanings(key: &str) -> &'static [&'static str] {
        dictionary_phf::verb_meanings(key).unwrap_or(&[])
    }

    /// Returns the Wiktionary definitions for an adjective **sense key** (e.g. `"bad3"`).
    /// Requires the `dictionary` feature.
    #[cfg(feature = "dictionary")]
    pub fn adj_meanings(key: &str) -> &'static [&'static str] {
        dictionary_phf::adj_meanings(key).unwrap_or(&[])
    }

    /// Capitalizes the first letter of a string.
    ///
    /// # Examples
    /// ```rust
    /// use english::English;
    ///
    /// assert_eq!(English::capitalize_first(""), "");
    /// assert_eq!(English::capitalize_first("house"), "House");
    /// ```
    pub fn capitalize_first(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}

/// Inflect a noun according to a numeric count.
///
/// # Examples
/// ```rust
/// use english::count;
///
/// assert_eq!(count("cat", 1), "cat");
/// assert_eq!(count("cat", 2), "cats");
/// ```
pub fn count(noun: &str, count: u32) -> String {
    if count == 1 {
        English::noun(noun, &Number::Singular)
    } else {
        English::noun(noun, &Number::Plural)
    }
}

/// Inflect a noun according to a numeric count and keep the number in the
/// output.
///
/// # Examples
/// ```rust
/// use english::count_with_number;
///
/// assert_eq!(count_with_number("cat", 1), "1 cat");
/// assert_eq!(count_with_number("cat", 2), "2 cats");
/// ```
pub fn count_with_number(noun: &str, amount: u32) -> String {
    format!("{} {}", amount, count(noun, amount))
}
