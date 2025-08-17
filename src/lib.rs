use core::fmt;

use english_core::EnglishCore;
pub use english_core::grammar::*;
mod noun_array;
use noun_array::*;
mod verb_array;
use verb_array::*;
mod adj_array;
use adj_array::*;
mod noun;
pub use noun::*;

fn strip_trailing_number(word: &str) -> Option<String> {
    if let Some(last_char) = word.chars().last() {
        if last_char.is_ascii_digit() {
            return Some(word[..word.len() - 1].to_string());
        }
    }
    None
}

/// Entry point for English inflection and morphology.
///
/// This struct provides high-level methods for handling English
/// nouns, verbs, adjectives, pronouns, and possessives.
/// It delegates irregular forms to internal lookup tables
/// and falls back on `EnglishCore` for regular inflection rules.
pub struct English {}
impl English {
    /// Inflects a noun into singular or plural form.
    ///
    /// Handles irregular nouns (e.g., `"child" -> "children"`) and
    /// falls back to regular pluralization rules when no override is found.
    /// Strips trailing numbers used for sense disambiguation (`"die2" -> "dice"`).
    ///
    /// # Examples
    /// ```rust
    /// assert_eq!(English::noun("cat", &Number::Plural), "cats");
    /// assert_eq!(English::noun("child", &Number::Plural), "children");
    /// assert_eq!(English::noun("die2", &Number::Plural), "dice");
    /// ```
    pub fn noun<T: Into<Noun>>(word: T, number: &Number) -> String {
        let noun: Noun = word.into();
        let base_word = strip_trailing_number(&noun.head).unwrap_or(noun.head.clone());

        let head_inflected = match number {
            Number::Singular => base_word,
            Number::Plural => {
                if let Some(x) = get_plural(&noun.head) {
                    x.to_string()
                } else {
                    EnglishCore::noun(&base_word, number)
                }
            }
        };
        format!(
            "{}{}{}",
            noun.specifier
                .as_ref()
                .map(|s| format!("{} ", s))
                .unwrap_or_default(),
            head_inflected,
            noun.complement
                .as_ref()
                .map(|c| format!(" {}", c))
                .unwrap_or_default()
        )
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
    /// assert_eq!(English::adj("fast", &Degree::Comparative), "faster");
    /// assert_eq!(English::adj("good", &Degree::Superlative), "best");
    /// assert_eq!(English::adj("fun", &Degree::Comparative), "more fun");
    /// ```
    pub fn adj(word: &str, degree: &Degree) -> String {
        let base_word = strip_trailing_number(word).unwrap_or(word.to_string());
        match degree {
            Degree::Positive => base_word.to_string(),
            Degree::Comparative => {
                if let Some((comp, _)) = get_adjective_forms(word) {
                    comp.to_string()
                } else {
                    EnglishCore::comparative(&base_word)
                }
            }
            Degree::Superlative => {
                if let Some((_, sup)) = get_adjective_forms(word) {
                    sup.to_string()
                } else {
                    EnglishCore::superlative(&base_word)
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
        let base_word = strip_trailing_number(word).unwrap_or(word.to_string());
        match get_verb_forms(word) {
            Some(wordik) => match (person, number, tense, form) {
                (_, _, _, Form::Infinitive) => {
                    return base_word.to_string();
                }

                (Person::Third, Number::Singular, Tense::Present, Form::Finite) => {
                    wordik.0.to_string()
                }
                (_, _, Tense::Present, Form::Finite) => {
                    return base_word.to_string();
                }
                (_, _, Tense::Present, Form::Participle) => wordik.2.to_string(),
                (_, _, Tense::Past, Form::Participle) => wordik.3.to_string(),

                (_, _, Tense::Past, Form::Finite) => wordik.1.to_string(),
            },
            None => EnglishCore::verb(&base_word, person, number, tense, form),
        }
    }
    /// Returns the correct English pronoun for the given grammatical features.
    ///
    /// # Examples
    /// ```rust
    /// assert_eq!(
    ///     English::pronoun(&Person::First, &Number::Singular, &Gender::Neutral, &Case::Nominative),
    ///     "I"
    /// );
    /// assert_eq!(
    ///     English::pronoun(&Person::Third, &Number::Singular, &Gender::Feminine, &Case::Nominative),
    ///     "she"
    /// );
    /// assert_eq!(
    ///     English::pronoun(&Person::Third, &Number::Plural, &Gender::Neutral, &Case::Nominative),
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
    /// assert_eq!(English::add_possessive("dog"), "dog's");
    /// assert_eq!(English::add_possessive("dogs"), "dogs'");
    /// ```
    pub fn add_possessive(word: &str) -> String {
        EnglishCore::add_possessive(word)
    }

    /// Returns a noun inflected according to the count. Wrapper around English::noun()
    ///
    /// # Examples
    /// ```rust
    /// assert_eq!(English::count("cat", 1), "cat");
    /// assert_eq!(English::count("cat", 2), "cats");
    /// ```
    pub fn count(word: &str, count: u32) -> String {
        if count == 1 {
            English::noun(word, &Number::Singular)
        } else {
            English::noun(word, &Number::Plural)
        }
    }

    /// Returns a noun inflected according to the count, preserves the number in output
    ///
    /// # Examples
    /// ```rust
    /// assert_eq!(English::count("cat", 1), "1 cat");
    /// assert_eq!(English::count("cat", 2), "2 cats");
    /// ```
    pub fn count_with_number(word: &str, count: u32) -> String {
        format!("{} {}", count, English::count(word, count))
    }
}
