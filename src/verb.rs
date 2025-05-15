use crate::grammar::*;
use Mood::*;
use Tense::*;
use VerbForm::*;

impl English {
    pub fn verb(word: &str, form: &Verb) -> String {
        English::regular_verb(word, form)
    }

    pub fn regular_verb(word: &str, form: &Verb) -> String {
        if word == "be" {
            return English::to_be(form).to_string();
        }
        match form {
            // Present simple 3rd person singular
            Verb {
                form: Finite,
                tense: Some(Present),
                mood: Some(Indicative),
                person: Person::Third,
                number: Number::Singular,
            } => format!("{}s", word),

            // Past tense (regular)
            Verb {
                form: Finite,
                tense: Some(Past),
                ..
            } => format!("{}ed", word),

            // Past participle
            Verb {
                form: Participle,
                tense: Some(Past),
                ..
            } => format!("{}ed", word),

            // Present participle
            Verb {
                form: Participle,
                tense: Some(Present),
                ..
            } => format!("{}ing", word),

            // Default fallback
            _ => word.to_string(),
        }
    }

    pub fn to_be(form: &Verb) -> &'static str {
        match form {
            // Present indicative
            Verb {
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
            Verb {
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
            Verb {
                form: Participle,
                tense: Some(Present),
                ..
            } => "being",

            // Past participle
            Verb {
                form: Participle,
                tense: Some(Past),
                ..
            } => "been",

            // Imperative "be"
            Verb {
                form: Finite,
                mood: Some(Imperative),
                ..
            } => "be",

            // Subjunctive present
            Verb {
                form: Finite,
                mood: Some(Subjunctive),
                tense: Some(Present),
                ..
            } => "be",

            // Subjunctive past
            Verb {
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
