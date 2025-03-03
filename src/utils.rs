use crate::grammar::*;
impl English {
    pub fn pair_match(word: &str, listik: &[(&str, &str)]) -> Option<String> {
        for (sing, plur) in listik {
            if sing == &word {
                return Some((*plur).into());
            }
        }
        None
    }
    pub fn replace_last_occurence(input: &str, pattern: &str, replacement: &str) -> String {
        if let Some(last_index) = input.rfind(pattern) {
            let (before_last, _after_last) = input.split_at(last_index);
            format!("{}{}", before_last, replacement)
        } else {
            input.into()
        }
    }

    pub fn starts_with_uppercase(word: &str) -> bool {
        word.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }
}
