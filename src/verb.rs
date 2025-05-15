use crate::grammar::*;

impl English {
    pub fn verb(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
    ) -> String {
        match word {
            "be" => {
                return English::to_be(person, number, tense, form).to_string();
            }
            _ => (),
        }
        match (person, number, tense, form) {
            (_, _, _, Form::Infinitive) => {
                return word.to_string();
            }

            (Person::Third, Number::Singular, Tense::Present, Form::Finite) => {
                return format!("{}{}", word, "s");
            }
            (_, _, Tense::Present, Form::Finite) => {
                return word.to_string();
            }

            (_, _, Tense::Past, Form::Finite | Form::Participle) => {
                return format!("{}{}", word, "ed");
            }

            (_, _, Tense::Present, Form::Participle) => {
                return format!("{}{}", word, "ing");
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
