pub struct English {}
pub enum Number {
    Singular,
    Plural,
}

const IRREGULAR_NOUNS: &[(&str, &str)] = &[("blin", "bliny")];
const PRONOUNS: &[(&str, &str)] = &[
    ("I", "we"),
    ("me", "us"),
    ("myself", "ourselves"),
    ("you", "y'all"),
    ("I", "we"),
    ("I", "we"),
    ("I", "we"),
];
const INDECLINEABLE_NOUNS: &[&str] = &["chassis"];

impl English {
    pub fn noun(word: &str, number: Number) -> String {
        match number {
            Number::Singular => return word.to_string(),
            Number::Plural => return English::pluralize_noun(word),
        }
    }

    fn pair_match(word: &str, listik: &[(&str, &str)]) -> Option<String> {
        for (sing, plur) in listik {
            if sing == &word {
                return Some((*plur).into());
            }
        }
        None
    }

    fn irregular_nouns(word: &str) -> Option<String> {
        if let Some(pronoun) = English::pair_match(word, PRONOUNS) {
            return Some(pronoun);
        }
        if let Some(pronoun) = English::pair_match(word, IRREGULAR_NOUNS) {
            return Some(pronoun);
        }

        None
    }
    fn starts_with_uppercase(word: &str) -> bool {
        word.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }
    fn is_indeclineable_nationality(word: &str) -> bool {
        English::starts_with_uppercase(word) && word.ends_with("ese")
    }
    fn non_declineable(word: &str) -> Option<String> {
        if word.ends_with("fish")
            || word.ends_with("ois")
            || word.ends_with("sheep")
            || word.ends_with("deer")
            || word.ends_with("pox")
            || word.ends_with("itis")
            || English::is_indeclineable_nationality(word)
            || INDECLINEABLE_NOUNS.contains(&word)
        {
            return Some(word.into());
        }
        None
    }

    fn pluralize_noun(word: &str) -> String {
        if let Some(irr) = English::irregular_nouns(word) {
            return irr;
        }
        if let Some(irr) = English::non_declineable(word) {
            return irr;
        }
        format!("{}{}", word, "s")
    }
}
