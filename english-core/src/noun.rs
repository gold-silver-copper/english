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
    //  ("chassis", "chassis"),
    //  ("sheep", "sheep"),
    ("mouse", "mice"),
    // ("louse", "lice"),
    ("tooth", "teeth"),
    ("goose", "geese"),
    ("trix", "trices"),
    ("fish", "fish"),
    ("deer", "deer"),
    ("itis", "itis"),
    ("foot", "feet"),
    ("zoon", "zoa"),
    ("ese", "ese"),
    ("man", "men"),
    ("pox", "pox"),
    ("ois", "ois"),
    ("cis", "ces"),
    ("sis", "ses"),
    ("xis", "xes"),
    ("eau", "eaux"),
    ("ieu", "ieux"),
    ("inx", "inges"),
    ("anx", "anges"),
    ("ynx", "ynges"),
    ("um", "a"),
    ("ch", "ches"),
    ("sh", "shes"),
    ("ay", "ays"),
    //  ("uy", "uys"),
    ("oy", "oys"),
    ("ey", "eys"),
    ("x", "xes"),
    //  ("a", "ae"),
    ("s", "ses"),
    ("y", "ies"),
    ("f", "ves"),
];
