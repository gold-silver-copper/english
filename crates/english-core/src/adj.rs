use crate::EnglishCore;
use crate::grammar::*;

impl EnglishCore {
    pub fn adjective(word: &str, degree: &Degree) -> String {
        match degree {
            Degree::Positive => word.to_string(),
            Degree::Comparative => Self::comparative(word),
            Degree::Superlative => Self::superlative(word),
        }
    }
    pub fn superlative(word: &str) -> String {
        match Self::degree_stem(word) {
            Some(stem) => format!("{stem}est"),
            None => format!("most {word}"),
        }
    }
    pub fn comparative(word: &str) -> String {
        match Self::degree_stem(word) {
            Some(stem) => format!("{stem}er"),
            None => format!("more {word}"),
        }
    }

    /// Decide whether an adjective takes suffixal degree forms (`-er`/`-est`)
    /// and return the adjusted stem, or `None` for periphrastic (`more`/`most`).
    ///
    /// Standard English inflection heuristic:
    /// - up to two syllables ending in consonant+`y`: `-y` -> `-i` (`happy` ->
    ///   `happier`, `dry` -> `drier`);
    /// - one syllable ending in `e`: drop nothing, bare `-r`/`-st` via an `e`-less
    ///   stem (`nice` -> `nicer`, `large` -> `larger`);
    /// - one syllable ending consonant-vowel-consonant (final not `w`/`x`/`y`):
    ///   double the final consonant (`big` -> `bigger`, `hot` -> `hotter`);
    /// - other one-syllable words: plain suffix (`fast` -> `faster`, `new` ->
    ///   `newer`);
    /// - everything else: periphrastic (`beautiful` -> `more beautiful`).
    ///
    /// This is the regular *rule*; per-word exceptions Wiktionary attests
    /// differently (`fun` -> `more fun`, `clever` -> `cleverer`) live in the
    /// `english` crate's lookup tables, which are consulted before this fallback
    /// and are regenerated against exactly this prediction.
    ///
    /// Known heuristic gap: a `y` between two vowels is counted as a vowel, so
    /// `loyal`/`royal` read as one syllable and the rule would suffix them
    /// (`loyaler`). They are attested periphrastic, so the table serves the
    /// correct `more loyal` (pinned by `english`'s `rule_regression_guards`); a
    /// bare `loyal` inflected by rule alone would be wrong.
    fn degree_stem(word: &str) -> Option<String> {
        if word.len() < 2 || !word.chars().all(|c| c.is_ascii_alphabetic()) {
            return None;
        }
        let lower = word.to_ascii_lowercase();
        let b = lower.as_bytes();
        let is_vowel = |c: u8| matches!(c, b'a' | b'e' | b'i' | b'o' | b'u');
        // Syllables ~= vowel groups (y counts as a vowel), minus a final silent e.
        let is_vowelish = |c: u8| is_vowel(c) || c == b'y';
        let mut syllables = 0u32;
        let mut prev_vowelish = false;
        for &c in b {
            let v = is_vowelish(c);
            if v && !prev_vowelish {
                syllables += 1;
            }
            prev_vowelish = v;
        }
        let len = b.len();
        // A final e after a consonant is silent ("large" is one syllable) — except in
        // a consonant+"le" ending, where it is syllabic ("simple" stays two syllables
        // and periphrastic; attested "simpler" comes from the tables, not the rule).
        let syllabic_le =
            len >= 3 && b[len - 1] == b'e' && b[len - 2] == b'l' && !is_vowelish(b[len - 3]);
        if b[len - 1] == b'e' && !is_vowelish(b[len - 2]) && syllables > 1 && !syllabic_le {
            syllables -= 1;
        }

        if b[len - 1] == b'y' && !is_vowel(b[len - 2]) && syllables <= 2 {
            // happy -> happi(er), dry -> dri(er)
            return Some(format!("{}i", &word[..word.len() - 1]));
        }
        if syllables != 1 {
            return None;
        }
        if b[len - 1] == b'e' {
            // nice -> nic(er): the stem drops the e, the suffix re-adds it
            return Some(word[..word.len() - 1].to_string());
        }
        if len >= 3
            && !is_vowelish(b[len - 1])
            && is_vowel(b[len - 2])
            && !is_vowelish(b[len - 3])
            && !matches!(b[len - 1], b'w' | b'x')
        {
            // consonant-vowel-consonant: big -> bigg(er)
            return Some(format!("{}{}", word, b[len - 1] as char));
        }
        Some(word.to_string())
    }
    pub fn pronoun(person: &Person, number: &Number, gender: &Gender, case: &Case) -> &'static str {
        match number {
            Number::Singular => match person {
                Person::First => match case {
                    Case::Nominative => "I",
                    Case::Accusative => "me",
                    Case::Reflexive => "myself",
                    Case::Possessive => "mine",
                    Case::PersonalPossessive => "my",
                },
                Person::Second => match case {
                    Case::Nominative => "you",
                    Case::Accusative => "you",
                    Case::Reflexive => "yourself",
                    Case::Possessive => "yours",
                    Case::PersonalPossessive => "your",
                },
                Person::Third => match gender {
                    Gender::Masculine => match case {
                        Case::Nominative => "he",
                        Case::Accusative => "him",
                        Case::Reflexive => "himself",
                        Case::Possessive => "his",
                        Case::PersonalPossessive => "his",
                    },
                    Gender::Feminine => match case {
                        Case::Nominative => "she",
                        Case::Accusative => "her",
                        Case::Reflexive => "herself",
                        Case::Possessive => "hers",
                        Case::PersonalPossessive => "her",
                    },
                    Gender::Neuter => match case {
                        Case::Nominative => "it",
                        Case::Accusative => "it",
                        Case::Reflexive => "itself",
                        Case::Possessive => "its",
                        Case::PersonalPossessive => "its",
                    },
                },
            },
            Number::Plural => match person {
                Person::First => match case {
                    Case::Nominative => "we",
                    Case::Accusative => "us",
                    Case::Reflexive => "ourselves",
                    Case::Possessive => "ours",
                    Case::PersonalPossessive => "our",
                },
                Person::Second => match case {
                    Case::Nominative => "you",
                    Case::Accusative => "you",
                    Case::Reflexive => "yourselves",
                    Case::Possessive => "yours",
                    Case::PersonalPossessive => "your",
                },
                Person::Third => match case {
                    Case::Nominative => "they",
                    Case::Accusative => "them",
                    Case::Reflexive => "themselves",
                    Case::Possessive => "theirs",
                    Case::PersonalPossessive => "their",
                },
            },
        }
    }
    //dog's -> dogs', child's -> children's, Mary's -> Marys'
    //  pub fn genitive_adjective(word: &str, number: &Number) -> String {}
}

#[cfg(test)]
mod tests {
    use super::EnglishCore;

    #[test]
    fn one_syllable_takes_er_est() {
        assert_eq!(EnglishCore::comparative("fast"), "faster");
        assert_eq!(EnglishCore::superlative("fast"), "fastest");
        assert_eq!(EnglishCore::comparative("cool"), "cooler");
        assert_eq!(EnglishCore::comparative("new"), "newer"); // final w never doubles
        assert_eq!(EnglishCore::comparative("slow"), "slower");
        assert_eq!(EnglishCore::comparative("coy"), "coyer"); // vowel+y is not -y inflection
    }

    #[test]
    fn cvc_doubles_the_final_consonant() {
        assert_eq!(EnglishCore::comparative("big"), "bigger");
        assert_eq!(EnglishCore::superlative("big"), "biggest");
        assert_eq!(EnglishCore::comparative("hot"), "hotter");
        assert_eq!(EnglishCore::comparative("sad"), "sadder");
    }

    #[test]
    fn final_e_merges_with_the_suffix() {
        assert_eq!(EnglishCore::comparative("nice"), "nicer");
        assert_eq!(EnglishCore::superlative("nice"), "nicest");
        assert_eq!(EnglishCore::comparative("large"), "larger"); // silent e: one syllable
        assert_eq!(EnglishCore::comparative("free"), "freer");
    }

    #[test]
    fn consonant_y_becomes_ier() {
        assert_eq!(EnglishCore::comparative("happy"), "happier");
        assert_eq!(EnglishCore::superlative("happy"), "happiest");
        assert_eq!(EnglishCore::comparative("dry"), "drier");
        assert_eq!(EnglishCore::comparative("early"), "earlier");
        assert_eq!(EnglishCore::comparative("hungry"), "hungrier");
    }

    #[test]
    fn longer_words_stay_periphrastic() {
        assert_eq!(EnglishCore::comparative("beautiful"), "more beautiful");
        assert_eq!(EnglishCore::superlative("beautiful"), "most beautiful");
        assert_eq!(EnglishCore::comparative("clever"), "more clever");
        assert_eq!(EnglishCore::comparative("narrow"), "more narrow");
        // -le is a syllabic e, not a silent one: two syllables -> periphrastic
        // (attested "simpler" belongs to the tables, not the rule).
        assert_eq!(EnglishCore::comparative("simple"), "more simple");
        // Non-alphabetic input is never suffixed.
        assert_eq!(EnglishCore::comparative("mp3"), "more mp3");
    }
}
