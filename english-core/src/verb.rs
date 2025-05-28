use crate::grammar::*;
use crate::EnglishCore;
impl EnglishCore {
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
                if word.ends_with("s")
                    || word.ends_with("z")
                    || word.ends_with("sh")
                    || word.ends_with("x")
                {
                    return format!("{}{}", word, "es");
                } else {
                    return format!("{}{}", word, "s");
                }
            }
            (_, _, Tense::Present, Form::Finite) => {
                return word.to_string();
            }
            (_, _, Tense::Present, Form::Participle) => {
                if word.ends_with("e") {
                    return EnglishCore::replace_last_occurence(word, "e", "ing");
                }
                if word.ends_with("p") {
                    return format!("{}{}", word, "ping");
                } else {
                    return format!("{}{}", word, "ing");
                }
            }

            (_, _, Tense::Past, _) => {
                if word.ends_with("e") {
                    return format!("{}{}", word, "d");
                }
                if word.ends_with("p") {
                    return format!("{}{}", word, "ped");
                } else {
                    return format!("{}{}", word, "ed");
                }
            }
        }
    }
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
