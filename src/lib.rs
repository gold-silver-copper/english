pub struct English {}
pub enum Number {
    Singular,
    Plural,
}

const IRREGULAR_NOUNS: &[(&str, &str)] = &[("cow", "cowzz")];

impl English {
    pub fn noun(word: &str, number: Number) -> String {
        match number {
            Number::Singular => return word.to_string(),
            Number::Plural => return English::pluralize_noun(word),
        }
    }
    fn irregular_nouns(word: &str) -> Option<String> {
        for (sing, plur) in IRREGULAR_NOUNS {
            if sing == &word {
                return Some((*plur).into());
            }
        }
        None
    }
    fn pluralize_noun(word: &str) -> String {
        if let Some(irr) = English::irregular_nouns(word) {
            return irr;
        }
        format!("{}{}", word, "s")
    }
}
