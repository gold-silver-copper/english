use crate::grammar::*;

impl English {
    pub fn verb(word: &str, person: &Person, number: &Number, tense: &Tense) -> String {
        match word {
            "be" => English::to_be(person, number, tense).to_string(),
            _ => English::regular_verb(word, person, number, tense),
        }
    }
    pub fn regular_verb(word: &str, person: &Person, number: &Number, tense: &Tense) -> String {
        if (person == &Person::Third
            && (number == &Number::Singular)
            && (tense == &Tense::SimplePresent))
        {
            return format!("{}{}", word, "s");
        } else if ((tense == &Tense::SimplePast) || (tense == &Tense::ParticiplePast)) {
            return format!("{}{}", word, "ed");
        } else if (tense == &Tense::ParticiplePresent) {
            return format!("{}{}", word, "ing");
        } else {
            return word.to_string();
        }
    }
    pub fn to_be(person: &Person, number: &Number, tense: &Tense) -> &'static str {
        match tense {
            Tense::SimplePresent => match number {
                Number::Singular => match person {
                    Person::First => "am",
                    Person::Second => "are",
                    Person::Third => "is",
                },
                Number::Plural => "are",
            },
            Tense::SimplePast => match number {
                Number::Singular => match person {
                    Person::First => "was",
                    Person::Second => "were",
                    Person::Third => "was",
                },
                Number::Plural => "were",
            },
            Tense::ParticiplePast => "been",
            Tense::ParticiplePresent => "being",
            Tense::ImperativePresent => "be",
            Tense::SubjunctivePresent => "be",
            Tense::SubjunctivePast => "were",
        }
    }
}
