pub struct English {}
pub enum Number {
    Singular,
    Plural,
}

const IRREGULAR_NOUNS: &[(&str, &str)] = &[("cow", "cowzz")];

impl English {
    pub fn noun(word: &str, number: Number) -> String {
        let try_irregular = English::irregular_nouns(word);
        if let Some(irr) = try_irregular {
            return irr;
        }

        match number {
            Number::Singular => return word.to_string(),
            Number::Plural => return format!("{}{}", word, "s"),
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
}
