use crate::grammar::*;
use crate::EnglishCore;

impl EnglishCore {
    pub fn noun(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => return word.to_string(),
            Number::Plural => return EnglishCore::pluralize_noun(word),
        }
    }
    pub fn possessive(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => {
                format!("{word}'s")
            }
            Number::Plural => {
                if word.ends_with('s') {
                    format!("{word}'") // Regular plural: dogs'
                } else {
                    format!("{word}'s") // Irregular plural: children’s
                }
            }
        }
    }

    fn irregular_suffix(word: &str) -> Option<String> {
        for (sing, plur) in IRREGULAR_SUFFIXES {
            if word.ends_with(sing) {
                return Some(EnglishCore::replace_last_occurence(word, sing, plur));
            }
        }
        None
    }

    pub fn pluralize_noun(word: &str) -> String {
        if let Some(irr) = EnglishCore::irregular_suffix(word) {
            return irr;
        }
        format!("{}{}", word, "s")
    }
}

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
    ("s", "ses"),
];
