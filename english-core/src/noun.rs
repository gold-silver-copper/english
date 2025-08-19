use crate::EnglishCore;
use crate::grammar::*;

impl EnglishCore {
    pub fn noun(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => return word.to_string(),
            Number::Plural => return EnglishCore::pluralize_noun(word),
        }
    }
    pub fn add_possessive(word: &str) -> String {
        if word.ends_with('s') {
            format!("{word}'") // Regular plural: dogs'
        } else {
            format!("{word}'s") // Irregular plural: children’s
        }
    }

    pub fn pluralize_noun(word: &str) -> String {
        if let Some(irr) = EnglishCore::iter_replace_last(word, IRREGULAR_SUFFIXES) {
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
    ("um", "a"),
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
    ("ay", "ays"),
    ("oy", "oys"),
    ("ey", "eys"),
    ("s", "ses"),
    ("y", "ies"),
    ("f", "ves"),
];
