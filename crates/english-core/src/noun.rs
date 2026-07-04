use crate::EnglishCore;
use crate::grammar::*;

impl EnglishCore {
    pub fn noun(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => word.to_string(),
            Number::Plural => EnglishCore::pluralize_noun(word),
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

// Suffix-class rules for the regular engine: unlike the `english` crate's
// per-word tables, these generalize over ENDINGS, so they also cover compounds
// the dump never lists ("dormouse" -> "dormice", "eigenclass" -> "eigenclasses").
//
// Layering contract with the `english` crate (guarded by its
// `tables_hold_only_what_the_rules_cannot_predict` test):
// - the tables are consulted first and hold per-word ATTESTED exceptions;
// - these rules are the fallback AND the extractor's "regular prediction", so any
//   word this list already handles is dropped from the tables at generation time.
// Consequence: adding a rule here shrinks the tables on the next `refresh-data`,
// and changing one without regenerating fails that test rather than silently
// double-representing (or contradicting) a word.
//
// Some entries are commented out because they did not pull their weight on
// accuracy vs. size when measured; re-evaluate with `cargo xtask accuracy`.
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
    // ("itis", "itis"),
    ("foot", "feet"),
    ("zoon", "zoa"),
    ("ese", "ese"),
    ("man", "men"),
    //("pox", "pox"),
    // ("ois", "ois"),
    //  ("cis", "ces"),
    ("sis", "ses"),
    ("xis", "xes"),
    //("eau", "eaux"),
    // ("ieu", "ieux"),
    // ("inx", "inges"),
    // ("anx", "anges"),
    //  ("ynx", "ynges"),
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
