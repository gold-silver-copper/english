//! Deterministic sense-key assignment.
//!
//! There is NO registry, lockfile, or identity here — key numbers are a pure
//! function of the current dump. For each `(lemma, pos)` the extractor collects
//! the attested inflection patterns, drops the one the regular rule engine
//! already produces (so the rule serves it at runtime), and this module numbers
//! the survivors:
//!
//! 1. sort them deterministically — standard senses before soft ones, then by
//!    emitted form signature lexicographically ([`Candidate::order_key`]);
//! 2. hand out suffixes from `1` (or `2` when a regular pattern was dropped, so
//!    the bare key is reserved for the rule engine): the first survivor gets the
//!    bare lemma or `lemma_2`, the next `lemma_3`, and so on ([`make_key`]).
//!
//! The only non-lexicographic tiebreak is standard-before-soft: a slang-only
//! sense (`bad` -> `badder`) must never take the bare key from a standard
//! suppletive sibling (`bad` -> `worse`). Everything else is the plain form sort.
//!
//! # Stability
//!
//! Keys are DETERMINISTIC but NOT immutable. The assignment is reproducible from
//! a given dump — reordering the dump's entries can never change it — but if
//! Wiktionary adds, removes, or edits a lemma's attested forms, the sort can
//! renumber its keys: a lexicographically earlier new variant deliberately shifts
//! the later ones up. That is the accepted tradeoff of a system with no
//! carry-forward state to maintain — "fairly stable", not frozen. There is no
//! human review, no override file, and no cross-version immutability gate.
//!
//! # Emitted key format (shared with the `english` runtime)
//!
//! Suffix 1 emits the bare lemma; `>= 2` emits `lemma_<n>`. The underscore is
//! unambiguous because extraction rejects any lemma containing `_`
//! ([`crate::helpers::word_is_proper`]), so a trailing `_<digits>` can only be a
//! sense suffix — digit-bearing words (`mp3`, `F16`) are never numbered keys. The
//! format itself lives in [`english_core::sense_key`], the single owner shared
//! with the runtime; [`make_key`]/[`split_key`] are this crate's thin re-exports.

/// A distinct attested inflection pattern for one lemma, awaiting a key.
#[derive(Debug, Clone, Default)]
pub struct Candidate {
    /// Emittable inflection columns in canonical order (noun: `[plural]`; verb:
    /// `[third, past, present_part, past_part]`; adj: `[comparative, superlative]`).
    pub forms: Vec<String>,
    /// True when EVERY contributing sense is soft (slang/nonstandard/...). Such a
    /// candidate sorts after standard siblings so it can never take the bare key
    /// from one (`bad` -> `worse`, not the slang `badder`).
    pub soft_sense: bool,
}

impl Candidate {
    pub fn new(forms: Vec<String>) -> Self {
        Candidate {
            forms,
            soft_sense: false,
        }
    }

    fn sig(&self) -> String {
        forms_sig(&self.forms)
    }

    /// The deterministic total order used to hand out suffixes: standard-before-
    /// soft, then the emitted form signature lexicographically. It is a pure
    /// function of the candidate's forms (and softness), so the output is
    /// invariant under any permutation of the dump's entry order.
    fn order_key(&self) -> (bool, String) {
        (self.soft_sense, self.sig())
    }
}

/// One emitted key plus the forms published under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub key: String,
    pub forms: Vec<String>,
}

/// Number a lemma's surviving candidates into deterministic keys.
///
/// `had_regular` records whether a regular-prediction-equal pattern was dropped
/// upstream (the rule engine serves it at runtime): when it was, the bare key is
/// reserved for the rule and numbering starts at `_2`; otherwise the first
/// candidate takes the bare lemma. Candidates are sorted by
/// [`Candidate::order_key`], so the result depends only on the set of surviving
/// forms — never on dump order.
pub fn assign(lemma: &str, mut candidates: Vec<Candidate>, had_regular: bool) -> Vec<Assignment> {
    candidates.sort_by_key(|c| c.order_key());
    let base = if had_regular { 2 } else { 1 };
    candidates
        .into_iter()
        .enumerate()
        .map(|(i, c)| Assignment {
            key: make_key(lemma, base + i as u32),
            forms: c.forms,
        })
        .collect()
}

/// The emitted key format lives in [`english_core::sense_key`] — the single
/// owner shared with the runtime, so the two can never disagree on which strings
/// are numbered keys. Re-exported here for the generator's callers.
pub use english_core::sense_key::make_key;

/// Decode an emitted key into `(base_lemma, suffix)`, or `None` for a bare key.
/// Wraps [`english_core::sense_key::split`] and parses the suffix to `u32`; an
/// overflowing digit run is a hard error (panic), never a silent coercion — a
/// coerced suffix would alias the bare key, and such a key can only come from a
/// corrupt generated table.
pub fn split_key(key: &str) -> Option<(&str, u32)> {
    let (base, digits) = english_core::sense_key::split(key)?;
    let suffix = digits
        .parse()
        .unwrap_or_else(|_| panic!("sense suffix overflows u32 in key {key:?}"));
    Some((base, suffix))
}

/// The canonical form-signature encoding (`forms.join("|")`). [`Candidate::sig`],
/// the extract-layer dedup, and `drop_regular` all funnel through this, so two
/// patterns merge exactly when they emit identical forms.
pub fn forms_sig(forms: &[String]) -> String {
    forms.join("|")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(forms: &[&str]) -> Candidate {
        Candidate::new(forms.iter().map(|s| s.to_string()).collect())
    }

    fn soft(forms: &[&str]) -> Candidate {
        let mut c = cand(forms);
        c.soft_sense = true;
        c
    }

    fn keys(a: &[Assignment]) -> Vec<String> {
        a.iter().map(|x| x.key.clone()).collect()
    }

    #[test]
    fn reordering_candidates_yields_identical_keys() {
        // The core determinism guarantee: dump order cannot change the output.
        let forward = assign("run", vec![cand(&["ran"]), cand(&["runned"])], false);
        let reversed = assign("run", vec![cand(&["runned"]), cand(&["ran"])], false);
        assert_eq!(forward, reversed);
        // Sorted by signature: "ran" < "runned", so recline-style "ran" is bare.
        assert_eq!(keys(&forward), vec!["run".to_string(), "run_2".to_string()]);
        assert_eq!(forward[0].forms, vec!["ran".to_string()]);
    }

    #[test]
    fn dropped_regular_reserves_the_bare_key() {
        // die: the regular "dies" was dropped (had_regular), so the single
        // irregular takes `_2` and the bare key falls through to the rule.
        let a = assign("die", vec![cand(&["dice"])], true);
        assert_eq!(keys(&a), vec!["die_2".to_string()]);
    }

    #[test]
    fn no_regular_gives_the_bare_key_to_the_first_sorted_candidate() {
        let a = assign("lie", vec![cand(&["lied"]), cand(&["lay"])], false);
        // "lay" < "lied": recline takes the bare key regardless of input order.
        assert_eq!(a[0].key, "lie");
        assert_eq!(a[0].forms, vec!["lay".to_string()]);
        assert_eq!(a[1].key, "lie_2");
        assert_eq!(a[1].forms, vec!["lied".to_string()]);
    }

    #[test]
    fn a_lexicographically_earlier_candidate_renumbers_the_later_ones() {
        // The accepted stability tradeoff: an earlier-sorting new variant shifts
        // the existing keys up by one, deterministically.
        let before = assign("x", vec![cand(&["m"]), cand(&["z"])], false);
        assert_eq!(before[0].forms, vec!["m".to_string()]); // x
        assert_eq!(before[1].forms, vec!["z".to_string()]); // x_2

        let after = assign("x", vec![cand(&["m"]), cand(&["z"]), cand(&["a"])], false);
        assert_eq!(
            keys(&after),
            vec!["x".to_string(), "x_2".to_string(), "x_3".to_string()]
        );
        assert_eq!(after[0].forms, vec!["a".to_string()]); // a is now the bare key
        assert_eq!(after[1].forms, vec!["m".to_string()]);
        assert_eq!(after[2].forms, vec!["z".to_string()]);
    }

    #[test]
    fn a_soft_sense_never_takes_the_bare_key() {
        // "badder" < "worse" lexicographically, but standard-before-soft keeps the
        // standard suppletive on the bare key and pushes the slang sense to `_2`.
        let a = assign(
            "bad",
            vec![soft(&["badder", "baddest"]), cand(&["worse", "worst"])],
            false,
        );
        assert_eq!(a[0].key, "bad");
        assert_eq!(a[0].forms, vec!["worse".to_string(), "worst".to_string()]);
        assert_eq!(a[1].key, "bad_2");
        assert_eq!(
            a[1].forms,
            vec!["badder".to_string(), "baddest".to_string()]
        );
    }

    #[test]
    fn split_key_round_trips_make_key() {
        assert_eq!(split_key("die_2"), Some(("die", 2)));
        assert_eq!(split_key("die"), None);
        assert_eq!(split_key("mp3"), None); // no underscore -> opaque
        assert_eq!(split_key("_2"), None); // empty base
        assert_eq!(split_key("x_"), None); // empty digits
        assert_eq!(split_key("x_2a"), None); // non-digit suffix
        for (lemma, suffix) in [("lie", 2u32), ("abide", 3), ("x", 17)] {
            let key = make_key(lemma, suffix);
            assert_eq!(split_key(&key), Some((lemma, suffix)));
        }
    }

    #[test]
    #[should_panic(expected = "overflows u32")]
    fn split_key_overflow_is_a_hard_error() {
        let _ = split_key("foo_4294967296");
    }
}
