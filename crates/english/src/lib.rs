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

/// Trims trailing ASCII digits — the raw sense-suffix strip. Suffixes are allocated
/// append-only and may grow past a single digit.
fn strip_trailing_number(word: &str) -> &str {
    word.trim_end_matches(|c: char| c.is_ascii_digit())
}

/// Resolve the base lemma for inflection. A trailing-digit run is treated as a
/// sense-disambiguation suffix **only when it actually resolves to a table key** —
/// either `word` itself is a key, or stripping the digits yields one. Otherwise the
/// input is opaque and returned unchanged, so digit-bearing words (`"mp3"`, `"F16"`,
/// `"co2"`) are never silently corrupted into `"mp"`/`"F"`/`"co"`. Numbered keys are
/// only ever generated for irregulars (which live in the tables), so a regular word
/// like `"cat2"` is not a real key and is left opaque rather than stripped to `"cat"`.
fn base_lemma<'a>(word: &'a str, is_key: impl Fn(&str) -> bool) -> &'a str {
    let stripped = strip_trailing_number(word);
    let base = if stripped != word && !stripped.is_empty() && (is_key(word) || is_key(stripped)) {
        stripped
    } else {
        word
    };
    debug_assert!(base.is_empty() == word.is_empty());
    base
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
        let base_word = base_lemma(word, |w| get_plural(w).is_some());

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
        let base_word = base_lemma(word, |w| get_adjective_forms(w).is_some());
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
        let base_word = base_lemma(word, |w| get_verb_forms(w).is_some());
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
