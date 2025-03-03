use crate::grammar::*;

impl English {
    pub fn to_be(person: &Person, number: &Number, tense: &Tense) -> &'static str {
        match tense {
            Tense::Present => match number {
                Number::Singular => match person {
                    Person::First => "am",
                    Person::Second => "are",
                    Person::Third => "is",
                },
                Number::Plural => "are",
            },
            Tense::Past => match number {
                Number::Singular => match person {
                    Person::First => "was",
                    Person::Second => "were",
                    Person::Third => "was",
                },
                Number::Plural => "were",
            },
        }
    }
}
