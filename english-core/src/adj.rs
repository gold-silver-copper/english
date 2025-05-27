use crate::grammar::*;

impl EnglishCore {
    pub fn adjective(word: &str, number: &Number) -> String {
        match number {
            Number::Singular => word.to_string(),
            Number::Plural => match word {
                "a" | "an" => "some".to_string(),
                "this" => "these".to_string(),
                "that" => "those".to_string(),
                _ => word.to_string(),
            },
        }
    }
    //dog's -> dogs', child's -> children's, Mary's -> Marys'
    //  pub fn genitive_adjective(word: &str, number: &Number) -> String {}
}
