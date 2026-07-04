//! The emitted sense-key format — the one string protocol shared by the table
//! generator (`extractor`) and the runtime (`english`).
//!
//! A lemma's senses/variants are numbered `lemma`, `lemma_2`, `lemma_3`, … Suffix
//! 1 (or 0) is the bare lemma; `>= 2` appends `_<n>`. The underscore is
//! unambiguous because extraction rejects any lemma containing `_`, so a trailing
//! `_<digits>` can only be a sense suffix — digit-bearing words (`mp3`, `F16`)
//! are never numbered keys.
//!
//! [`make_key`] and [`split`] are the sole encoder/decoder of this format;
//! keeping both here means the generator and runtime can never disagree on which
//! strings are numbered keys. Numeric interpretation of the suffix is left to the
//! caller: the runtime only needs the base lemma, while the generator parses the
//! digits to a `u32`.

/// Suffix 1 (or 0) emits the bare lemma; `>= 2` appends `_<n>`.
pub fn make_key(lemma: &str, suffix: u32) -> String {
    if suffix <= 1 {
        lemma.to_string()
    } else {
        format!("{lemma}_{suffix}")
    }
}

/// Decode an emitted key: `Some((base_lemma, digits))` when `key` carries a
/// `_<digits>` sense suffix, `None` for a bare key (or a non-key). The digits are
/// returned as a string — the caller decides whether/how to parse them.
pub fn split(key: &str) -> Option<(&str, &str)> {
    let (base, digits) = key.rsplit_once('_')?;
    if base.is_empty() || digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((base, digits))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_round_trips_make_key_and_rejects_non_keys() {
        for (lemma, suffix) in [("lie", 2u32), ("abide", 3), ("x", 17)] {
            let key = make_key(lemma, suffix);
            assert_eq!(split(&key), Some((lemma, suffix.to_string().as_str())));
        }
        assert_eq!(make_key("die", 1), "die"); // bare
        assert_eq!(make_key("die", 0), "die");
        assert_eq!(split("die"), None); // bare -> not a numbered key
        assert_eq!(split("mp3"), None); // no underscore
        assert_eq!(split("_2"), None); // empty base
        assert_eq!(split("x_"), None); // empty digits
        assert_eq!(split("x_2a"), None); // non-digit tail
        // Edge digits are accepted as-is; numeric interpretation is the caller's.
        assert_eq!(split("x_0"), Some(("x", "0")));
        assert_eq!(split("x_007"), Some(("x", "007")));
    }
}
