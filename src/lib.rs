pub struct English {}
pub enum Number {
    Singular,
    Plural,
}

impl English {
    pub fn noun(word: &str, number: Number) -> String {
        match number {
            Number::Singular => return word.to_string(),
            Number::Plural => return format!("{}{}", word, "s"),
        }
    }
}
