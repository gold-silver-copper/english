use crate::grammar::*;
use Mood::*;
use Tense::*;
use VerbForm::*;

impl English {
    pub fn verb(word: &str, form: &VerbFormSpec) -> String {
        match word {
            "be" => English::to_be(form).to_string(),
            _ => English::regular_verb(word, form),
        }
    }

    pub fn regular_verb(word: &str, form: &VerbFormSpec) -> String {
        match form {
            // Present simple 3rd person singular
            VerbFormSpec {
                form: Finite,
                tense: Some(Present),
                mood: Some(Indicative),
                person: Person::Third,
                number: Number::Singular,
            } => format!("{}s", word),

            // Past tense (regular)
            VerbFormSpec {
                form: Finite,
                tense: Some(Past),
                ..
            } => format!("{}ed", word),

            // Past participle
            VerbFormSpec {
                form: Participle,
                tense: Some(Past),
                ..
            } => format!("{}ed", word),

            // Present participle
            VerbFormSpec {
                form: Participle,
                tense: Some(Present),
                ..
            } => format!("{}ing", word),

            // Default fallback
            _ => word.to_string(),
        }
    }

    pub fn to_be(form: &VerbFormSpec) -> &'static str {
        match form {
            // Present indicative
            VerbFormSpec {
                form: Finite,
                tense: Some(Present),
                mood: Some(Indicative),
                number,
                person,
            } => match number {
                Number::Singular => match person {
                    Person::First => "am",
                    Person::Second => "are",
                    Person::Third => "is",
                },
                Number::Plural => "are",
            },

            // Past indicative
            VerbFormSpec {
                form: Finite,
                tense: Some(Past),
                mood: Some(Indicative),
                number,
                person,
            } => match number {
                Number::Singular => match person {
                    Person::First => "was",
                    Person::Second => "were",
                    Person::Third => "was",
                },
                Number::Plural => "were",
            },

            // Present participle
            VerbFormSpec {
                form: Participle,
                tense: Some(Present),
                ..
            } => "being",

            // Past participle
            VerbFormSpec {
                form: Participle,
                tense: Some(Past),
                ..
            } => "been",

            // Imperative "be"
            VerbFormSpec {
                form: Finite,
                mood: Some(Imperative),
                ..
            } => "be",

            // Subjunctive present
            VerbFormSpec {
                form: Finite,
                mood: Some(Subjunctive),
                tense: Some(Present),
                ..
            } => "be",

            // Subjunctive past
            VerbFormSpec {
                form: Finite,
                mood: Some(Subjunctive),
                tense: Some(Past),
                ..
            } => "were",

            // Fallback
            _ => "be",
        }
    }
}
