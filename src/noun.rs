use crate::grammar::*;

//These are most of the irregular suffixes, not counted so far are wolves,potatoes,compound words
const IRREGULAR_SUFFIXES: &[(&str, &str)] = &[
    ("fish", "fish"),
    ("ois", "ois"),
    ("sheep", "sheep"),
    ("deer", "deer"),
    ("pox", "pox"),
    ("itis", "itis"),
    ("chassis", "chassis"),
    ("ese", "ese"),
    ("man", "men"),
    ("mouse", "mice"),
    ("louse", "lice"),
    ("tooth", "teeth"),
    ("goose", "geese"),
    ("foot", "feet"),
    ("zoon", "zoa"),
    ("cis", "ces"),
    ("sis", "ses"),
    ("xis", "xes"),
    ("trix", "trices"),
    ("eau", "eaux"),
    ("ieu", "ieux"),
    ("inx", "inges"),
    ("anx", "anges"),
    ("ynx", "ynges"),
    ("ex", "exes"),
    ("ch", "ches"),
    ("sh", "shes"),
    ("ss", "sses"),
];

impl English {
    pub fn noun(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => return word.to_string(),
            Number::Plural => return English::pluralize_noun(word),
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

    fn irregular_suffix(word: &str) -> Option<String> {
        for (sing, plur) in IRREGULAR_SUFFIXES {
            if word.ends_with(sing) {
                return Some(English::replace_last_occurence(word, sing, plur));
            }
        }
        None
    }

    fn pluralize_noun(word: &str) -> String {
        if let Some(irr) = English::irregular_suffix(word) {
            return irr;
        }
        format!("{}{}", word, "s")
    }
}
