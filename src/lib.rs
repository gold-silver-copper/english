use core::fmt;

use english_core::EnglishCore;
pub use english_core::grammar::*;

mod insane_array;
use insane_array::*;
mod verb_array;
use verb_array::*;
mod adj_array;
use adj_array::*;
mod noun;
pub use noun::*;
mod verb;
pub use verb::*;

pub fn strip_trailing_number(word: &str) -> (String, Option<u32>) {
    let mut chars = word.char_indices().rev();
    let mut end = word.len();

    for (idx, ch) in &mut chars {
        if ch.is_ascii_digit() {
            end = idx;
        } else {
            break;
        }
    }

    if end == word.len() {
        // No trailing number
        return (word.to_string(), None);
    }

    let number_str = &word[end..];
    let number = number_str.parse::<u32>().ok();
    let stripped = word[..end].to_string();

    (stripped, number)
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
        let (base_word, num) = strip_trailing_number(&noun.head);
        let mut num = num.unwrap_or(1);

        let head_inflected = match number {
            Number::Singular => base_word.clone(),
            Number::Plural => {
                // Get last char of base_word
                let last_char = base_word
                    .chars()
                    .last()
                    .unwrap_or_default()
                    .to_ascii_lowercase();

                INSANE_MAP
                    .iter()
                    .find(|(letter, _)| letter.chars().next().unwrap_or_default() == last_char)
                    .and_then(|(_, rules)| {
                        rules.iter().find_map(|(sing, plural)| {
                            if base_word.ends_with(sing) {
                                num -= 1;
                                if num == 0 {
                                    Some(EnglishCore::replace_last_occurence(
                                        &base_word, sing, plural,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or_else(|| EnglishCore::noun(&base_word, number))
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
        let (base_word, _) = strip_trailing_number(word);
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
    pub fn verb<T: Into<Verb>>(
        wordish: T,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
    ) -> String {
        let verb: Verb = wordish.into();
        let (base_word, _) = strip_trailing_number(&verb.head);

        // Conjugate the head verb
        let conjugated_head = match get_verb_forms(&verb.head) {
            Some(wordik) => match (person, number, tense, form) {
                (_, _, _, Form::Infinitive) => base_word.clone(),
                (Person::Third, Number::Singular, Tense::Present, Form::Finite) => {
                    wordik.0.to_string()
                }
                (_, _, Tense::Present, Form::Finite) => base_word.clone(),
                (_, _, Tense::Present, Form::Participle) => wordik.2.to_string(),
                (_, _, Tense::Past, Form::Participle) => wordik.3.to_string(),
                (_, _, Tense::Past, Form::Finite) => wordik.1.to_string(),
            },
            None => EnglishCore::verb(&base_word, person, number, tense, form),
        };
        // Reassemble phrasal verb with particle
        if let Some(particle) = verb.particle {
            format!("{} {}", conjugated_head, particle)
        } else {
            conjugated_head
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
    pub fn count<T: Into<Noun>>(word: T, count: u32) -> String {
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
    pub fn count_with_number<T: Into<Noun>>(word: T, count: u32) -> String {
        format!("{} {}", count, English::count(word, count))
    }
}
