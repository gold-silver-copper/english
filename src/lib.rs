use core::fmt;

pub use english_core::grammar::*;
pub use english_core::EnglishCore;
mod nounsiki;
pub use nounsiki::*;
mod verbsiki;
pub use verbsiki::*;
mod sentence;
pub struct English {}
impl English {
    pub fn noun(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => {
                return word.to_string();
            }
            Number::Plural => {
                if let Some(x) = get_plural(word) {
                    return x.to_string();
                } else {
                    return EnglishCore::noun(word, number);
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
        match get_verb_forms(word) {
            Some(wordik) => match (person, number, tense, form) {
                (_, _, _, Form::Infinitive) => {
                    return word.to_string();
                }

                (Person::Third, Number::Singular, Tense::Present, Form::Finite) => {
                    wordik.0.to_string()
                }
                (_, _, Tense::Present, Form::Finite) => {
                    return word.to_string();
                }
                (_, _, Tense::Present, Form::Participle) => wordik.2.to_string(),
                (_, _, Tense::Past, Form::Participle) => wordik.3.to_string(),

                (_, _, Tense::Past, Form::Finite) => wordik.1.to_string(),
            },
            None => EnglishCore::verb(word, person, number, tense, form),
        }
    }
    pub fn pronoun(person: &Person, number: &Number, gender: &Gender, case: &Case) -> &'static str {
        EnglishCore::pronoun(person, number, gender, case)
    }
    pub fn add_possessive(word: &str, number: &Number) -> String {
        EnglishCore::add_possessive(word, number)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Noun {
    pub word: String,
    pub number: Number,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Verb {
    pub word: String,
    pub person: Person,
    pub tense: Tense,
    pub form: Form,
}

impl fmt::Display for Noun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", English::noun(&self.word, &self.number))
    }
}

/*
impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            English::verb(&self.word, &self.person, &self.tense, &self.form)
        )
    }
}

*/
