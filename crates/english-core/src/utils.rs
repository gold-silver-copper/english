//! Small string helpers shared by the rule modules.

use crate::EnglishCore;
impl EnglishCore {
    /// Exact-word lookup in a `(word, replacement)` list. Unlike
    /// [`EnglishCore::iter_replace_last`], this matches the WHOLE word — used
    /// where suffix generalization would be wrong.
    pub fn pair_match(word: &str, list: &[(&str, &str)]) -> Option<String> {
        list.iter()
            .find(|(sing, _)| *sing == word)
            .map(|(_, plur)| plur.to_string())
    }

    /// Replace the LAST occurrence of `pattern` with `replacement`, keeping
    /// everything before it. Combined with an `ends_with` check by the caller,
    /// this rewrites a word's suffix while preserving any compound prefix
    /// (`dormouse` + (`mouse` -> `mice`) = `dormice`).
    pub fn replace_last_occurrence(input: &str, pattern: &str, replacement: &str) -> String {
        if let Some(last_index) = input.rfind(pattern) {
            let (before_last, _after_last) = input.split_at(last_index);
            format!("{}{}", before_last, replacement)
        } else {
            input.into()
        }
    }
    /// Apply the FIRST suffix-class rule whose suffix matches the end of the
    /// word. Order in `pairs` is load-bearing: more specific suffixes must be
    /// listed before their own suffixes' suffixes (`"sis" -> "ses"` must
    /// precede `"s" -> "ses"`, `"ay" -> "ays"` must precede `"y" -> "ies"`),
    /// because matching stops at the first hit. Reordering a list silently
    /// changes predictions — and therefore what the extractor tables — so any
    /// edit here requires a regeneration (see the crate docs).
    pub fn iter_replace_last(word: &str, pairs: &[(&str, &str)]) -> Option<String> {
        for (sing, plur) in pairs {
            if word.ends_with(sing) {
                return Some(EnglishCore::replace_last_occurrence(word, sing, plur));
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
