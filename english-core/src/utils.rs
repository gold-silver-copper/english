use crate::EnglishCore;
use crate::grammar::*;
impl EnglishCore {
    pub fn pair_match(word: &str, listik: &[(&str, &str)]) -> Option<String> {
        listik
            .iter()
            .find(|(sing, _)| *sing == word)
            .map(|(_, plur)| plur.to_string())
    }

    pub fn replace_last_occurence(input: &str, pattern: &str, replacement: &str) -> String {
        if let Some(last_index) = input.rfind(pattern) {
            let (before_last, _after_last) = input.split_at(last_index);
            format!("{}{}", before_last, replacement)
        } else {
            input.into()
        }
    }
    pub fn iter_replace_last(word: &str, pairs: &[(&str, &str)]) -> Option<String> {
        for (sing, plur) in pairs {
            if word.ends_with(sing) {
                return Some(EnglishCore::replace_last_occurence(word, sing, plur));
            }
        }
        None
    }

    pub fn starts_with_uppercase(word: &str) -> bool {
        word.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }
}
