use crate::grammar::*;
use crate::EnglishCore;

impl EnglishCore {
    pub fn adjective(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => word.to_string(),
            Number::Plural => match word {
                "a" | "an" => "some".to_string(),
                "this" => "these".to_string(),
                "that" => "those".to_string(),
                _ => word.to_string(),
            },
        }
    }
    pub fn pronoun(person: &Person, number: &Number, gender: &Gender, case: &Case) -> &'static str {
        match number {
            Number::Singular => match person {
                Person::First => match case {
                    Case::Nominative => "I",
                    Case::Accusative => "me",
                    Case::Reflexive => "myself",
                    Case::Possessive => "mine",
                    Case::PersonalPossesive => "my",
                },
                Person::Second => match case {
                    Case::Nominative => "you",
                    Case::Accusative => "you",
                    Case::Reflexive => "yourself",
                    Case::Possessive => "yours",
                    Case::PersonalPossesive => "your",
                },
                Person::Third => match gender {
                    Gender::Masculine => match case {
                        Case::Nominative => "he",
                        Case::Accusative => "him",
                        Case::Reflexive => "himself",
                        Case::Possessive => "his",
                        Case::PersonalPossesive => "his",
                    },
                    Gender::Feminine => match case {
                        Case::Nominative => "she",
                        Case::Accusative => "her",
                        Case::Reflexive => "herself",
                        Case::Possessive => "hers",
                        Case::PersonalPossesive => "her",
                    },
                    Gender::Neuter => match case {
                        Case::Nominative => "it",
                        Case::Accusative => "it",
                        Case::Reflexive => "itself",
                        Case::Possessive => "its",
                        Case::PersonalPossesive => "its",
                    },
                },
            },
            Number::Plural => match person {
                Person::First => match case {
                    Case::Nominative => "we",
                    Case::Accusative => "us",
                    Case::Reflexive => "ourselves",
                    Case::Possessive => "ours",
                    Case::PersonalPossesive => "our",
                },
                Person::Second => match case {
                    Case::Nominative => "you",
                    Case::Accusative => "you",
                    Case::Reflexive => "yourselves",
                    Case::Possessive => "yours",
                    Case::PersonalPossesive => "your",
                },
                Person::Third => match case {
                    Case::Nominative => "they",
                    Case::Accusative => "them",
                    Case::Reflexive => "themselves",
                    Case::Possessive => "theirs",
                    Case::PersonalPossesive => "their",
                },
            },
        }
    }
    //dog's -> dogs', child's -> children's, Mary's -> Marys'
    //  pub fn genitive_adjective(word: &str, number: &Number) -> String {}
}
