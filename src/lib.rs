use core::fmt;

use english_core::EnglishCore;
pub use english_core::grammar::*;
mod nounsiki;
pub use nounsiki::*;
mod verbsiki;
pub use verbsiki::*;
mod adjiki;
pub use adjiki::*;

fn strip_trailing_number(word: &str) -> Option<String> {
    if let Some(last_char) = word.chars().last() {
        if last_char.is_ascii_digit() {
            return Some(word[..word.len() - 1].to_string());
        }
    }
    None
}

pub struct English {}
impl English {
    pub fn noun(word: &str, number: &Number) -> String {
        let base_word = strip_trailing_number(word).unwrap_or(word.to_string());

        match number {
            Number::Singular => base_word,
            Number::Plural => {
                if let Some(x) = get_plural(word) {
                    x.to_string()
                } else {
                    EnglishCore::noun(&base_word, number)
                }
            }
        }
    }

    pub fn adj(word: &str, degree: &Degree) -> String {
        match degree {
            Degree::Positive => word.to_string(),
            Degree::Comparative => {
                if let Some((comp, _)) = get_adjective_forms(word) {
                    comp.to_string()
                } else {
                    EnglishCore::comparative(word)
                }
            }
            Degree::Superlative => {
                if let Some((_, sup)) = get_adjective_forms(word) {
                    sup.to_string()
                } else {
                    EnglishCore::superlative(word)
                }
            }
        }
    }

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
    pub fn pronoun(person: &Person, number: &Number, gender: &Gender, case: &Case) -> &'static str {
        EnglishCore::pronoun(person, number, gender, case)
    }
    pub fn add_possessive(word: &str) -> String {
        EnglishCore::add_possessive(word)
    }
}
