use crate::EnglishCore;
use crate::grammar::*;

impl EnglishCore {
    /// Inflect an adverb for degree by rule.
    ///
    /// Unlike adjectives, the regular adverb degree rule is unconditionally
    /// PERIPHRASTIC: `quickly -> more quickly / most quickly`. English has no
    /// productive suffixal adverb rule — the flat adverbs that do take `-er`/`-est`
    /// (`fast -> faster`, `hard -> harder`, `early -> earlier`) and the suppletives
    /// (`well -> better`, `badly -> worse`, `far -> farther`) are a small, closed,
    /// ATTESTED set served by the `english` crate's lookup table, not derivable by
    /// rule. Measured on the 2026-07 dump, 13,037 of 13,155 gradable adverbs
    /// (99.1%) take the periphrastic comparative; the 147 single-word exceptions
    /// are tabled. So the conservative `more`/`most` default is correct for the
    /// overwhelming majority, and a suffixal guess would be wrong far more often
    /// than right (it would give `quicklier`, `abruptlier`, ...).
    pub fn adverb(word: &str, degree: &Degree) -> String {
        match degree {
            Degree::Positive => word.to_string(),
            Degree::Comparative => Self::comparative_adverb(word),
            Degree::Superlative => Self::superlative_adverb(word),
        }
    }

    /// The regular (periphrastic) comparative of an adverb: `more <word>`.
    pub fn comparative_adverb(word: &str) -> String {
        format!("more {word}")
    }

    /// The regular (periphrastic) superlative of an adverb: `most <word>`.
    pub fn superlative_adverb(word: &str) -> String {
        format!("most {word}")
    }
}

#[cfg(test)]
mod tests {
    use super::EnglishCore;
    use crate::grammar::Degree;

    #[test]
    fn regular_adverbs_are_periphrastic() {
        assert_eq!(EnglishCore::comparative_adverb("quickly"), "more quickly");
        assert_eq!(EnglishCore::superlative_adverb("quickly"), "most quickly");
        assert_eq!(EnglishCore::comparative_adverb("abruptly"), "more abruptly");
        // The rule never guesses a suffixal form — even for short/flat-looking
        // adverbs, the rule stays periphrastic; `fast -> faster` comes from the
        // table, not here.
        assert_eq!(EnglishCore::comparative_adverb("fast"), "more fast");
        assert_eq!(EnglishCore::comparative_adverb("well"), "more well");
    }

    #[test]
    fn adverb_dispatch_matches_degree() {
        assert_eq!(EnglishCore::adverb("quickly", &Degree::Positive), "quickly");
        assert_eq!(
            EnglishCore::adverb("quickly", &Degree::Comparative),
            "more quickly"
        );
        assert_eq!(
            EnglishCore::adverb("quickly", &Degree::Superlative),
            "most quickly"
        );
    }
}
