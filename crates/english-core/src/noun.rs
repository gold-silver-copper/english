use crate::EnglishCore;
use crate::grammar::*;

impl EnglishCore {
    pub fn noun(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => word.to_string(),
            Number::Plural => EnglishCore::pluralize_noun(word),
        }
    }
    /// Add a possessive suffix using the crate's simple surface rule.
    ///
    /// Any word ending in `s` gets a bare apostrophe (`dogs'`, but also
    /// `bus'`/`James'`); everything else gets `'s`. This intentionally does not
    /// encode a style guide preference for singular names ending in `s`.
    pub fn add_possessive(word: &str) -> String {
        if word.ends_with('s') {
            format!("{word}'") // Regular plural: dogs'
        } else {
            format!("{word}'s") // Irregular plural: children's
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
//
// Several ACTIVE entries over-generalize and look like bugs (`f -> ves` gives
// `chief -> chieves`; `um -> a` gives `album -> alba`; `man -> men` gives
// `human -> humen`), but they are a deliberate, measured tradeoff, NOT safe to
// "fix" by removal. Each is a two-sided coin the dump cannot call: `-f` words
// attest both `selves` (standard) and `roofs`/`rooves`, so `f -> ves` serves
// self/shelf/leaf correctly while getting roof/staff wrong, and dropping it flips
// exactly those errors onto the more-common words. Worse, dropping it makes the
// rule predict `selfs`/`leafs` — which are plainly attested (Toronto Maple
// `Leafs`) — so they RESERVE the bare key (`extractor::LemmaAcc::drop_regular`)
// and demote `selves`/`leaves` to a `_<n>` key. Measured on the 2026-07 dump,
// removing `f -> ves` regressed ~78 common nouns for zero net accuracy. The
// `english` crate's `rule_regression_guards` test pins the vulnerable words.
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
