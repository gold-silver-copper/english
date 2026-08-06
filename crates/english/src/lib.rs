//! English inflection backed by Wiktionary-derived lookup tables with a
//! rule-engine fallback.
//!
//! # Architecture
//!
//! Every query follows the same two-tier shape: consult the generated PHF
//! tables (`generated/*_phf.rs`, compiled in via `include!`) for an attested
//! exception first, and fall back to [`EnglishCore`]'s regular rules otherwise.
//! The tables and the rules are NOT independent: the extractor drops any
//! candidate the rules already predict, so the tables hold exactly the attested
//! exceptions and nothing else. Changing a rule in `english-core` therefore
//! requires regenerating the tables (`cargo xtask refresh-data`). Two dump-free
//! tests partially guard this: the `rule_table_sync` test below catches a rule
//! change that makes an existing bare table row redundant, and english-core's
//! `regular_rules_golden` test catches one that breaks the fallback for un-tabled
//! regular words. Neither is exhaustive — `cargo xtask accuracy` (with the dump)
//! is the authoritative check after a rule change.
//!
//! # Sense-numbered keys
//!
//! Homograph senses and attested variants are published as `_<n>`-suffixed
//! keys (`die_2` -> *dice*). The underscore is unambiguous: the extractor never
//! admits a lemma containing `_`, so a trailing `_<digits>` can only be a sense
//! suffix — digit-bearing words (`mp3`, `F16`, `F2`) can never collide with one.
//! Adjacent-digit keys (`die2`) are deliberately not parsed: they are
//! indistinguishable from ordinary digit-bearing input.
//!
//! Key numbers are DETERMINISTIC but NOT immutable. They are assigned by a pure
//! sort of each lemma's emitted forms (see `extractor::assign`): the bare key
//! goes to the standard sense with the lexicographically-smallest signature (or
//! is reserved for the rule engine when a regular form is attested), and the
//! rest number upward. Regenerating from a newer Wiktionary dump can therefore
//! renumber a lemma's `_<n>` keys if its attested forms change — there is no
//! lockfile or frozen identity. What is stable is the *set* of forms a lemma
//! exposes and the rule/table layering below.
//!
//! # Lookup semantics (invariants shared by [`English::noun`]/[`English::verb`]/[`English::adj`]/[`English::adverb`])
//!
//! 1. Case-insensitive convenience: Title-case and ALL-CAPS input hit the
//!    all-lowercase tables with the casing restored on the value; regular-rule
//!    fallbacks get the identical treatment via the internal `rule_with_case`
//!    helper, so `HOT -> HOTTER` exactly as `GOOD -> BEST`. Mixed case is never
//!    guessed at (`McDonald` falls through as `McDonalds`). Known acronym
//!    homographs of irregular rows are excluded from the case-folded fallback
//!    (`US` inflects by rule, never through the `us` row — see
//!    `ACRONYM_SHADOWED`). This is an ergonomic default, not semantic proper-noun
//!    detection; callers that care about acronyms, surnames, or house style should
//!    normalize casing before/after calling this crate.
//! 2. Base-lemma agreement: when a `_<n>` suffix strips (the word or its base
//!    is a table key), EVERY code path treats the input as that base lemma —
//!    lookups fall back `get_*(word)` then `get_*(base)` so `child_2` inflects
//!    exactly like `child` instead of mixing table singulars with rule plurals.
//! 3. Opaqueness: input that resolves to no key inflects by rule on the whole
//!    string, unchanged (`cat_2` -> `cat_2s`).

use english_core::EnglishCore;
pub use english_core::grammar::*;

mod noun_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/noun_phf.rs"
    ));
}
use noun_phf::*;
mod adj_phf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/adj_phf.rs"));
}
use adj_phf::*;
mod verb_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/verb_phf.rs"
    ));
}
use verb_phf::*;
mod adverb_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/adverb_phf.rs"
    ));
}
use adverb_phf::*;

/// The base of a canonical sense-suffixed key: sense suffixes are `_<digits>`
/// (underscore plus a number, e.g. `die_2`). Returns `None` when the word carries
/// no canonical suffix.
///
/// The underscore makes this unambiguous: the extractor never emits a lemma
/// containing `_` (such words fail its character filter), so a trailing
/// `_<digits>` can ONLY be a sense suffix. Digit-bearing words (`"mp3"`, `"F16"`,
/// `"F2"`, `"co2"`) have no underscore and are therefore never mangled.
///
/// Decoding goes through [`english_core::sense_key::split`] — the single owner of
/// the key format, shared with the extractor's generator — so the runtime and the
/// generator can never disagree on which strings are numbered keys.
fn canonical_sense_suffix_base(word: &str) -> Option<&str> {
    english_core::sense_key::split(word).map(|(base, _)| base)
}

/// Resolve the base lemma for inflection. A `_<digits>` sense suffix is honored
/// **only when it resolves to a table key** — either `word` itself is a key, or
/// the base is one. Otherwise the input is opaque and returned unchanged
/// (`"cat_2"` for a regular lemma is not a real key and pluralizes opaquely).
///
/// Invariant for callers: whenever this strips (returns the base), every code path
/// must treat the input as that base lemma — table lookups fall back to the base
/// (`get_*(word)` then `get_*(base)`), so `"child_2"` inflects exactly like
/// `"child"` rather than mixing stripped singulars with regular-rule plurals.
fn base_lemma(word: &str, is_key: impl Fn(&str) -> bool) -> &str {
    // Strip to `base` only when the `_<digits>` suffix resolves to a real table
    // key (the word itself or its base); otherwise the input is opaque. Only the
    // canonical `_<n>` spelling is parsed — the old adjacent-digit form (`die2`)
    // was retired at the data-major-2 re-key (see the crate docs).
    match canonical_sense_suffix_base(word) {
        Some(base) if is_key(word) || is_key(base) => base,
        _ => word,
    }
}

/// How a capitalized input's casing is restored onto a lowercase table value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseStyle {
    /// Lowercase / mixed-case input: the table value is returned as-is.
    AsIs,
    /// Title-case input (`Child`): capitalize the first letter of the output.
    Title,
    /// All-caps input (`CHILD`, more than one letter): uppercase the output.
    Upper,
}

fn case_style(word: &str) -> CaseStyle {
    let Some(first) = word.chars().next() else {
        return CaseStyle::AsIs;
    };
    if !first.is_uppercase() {
        return CaseStyle::AsIs;
    }
    let letters = word.chars().filter(|c| c.is_alphabetic());
    if letters.clone().count() > 1 && letters.clone().all(|c| !c.is_lowercase()) {
        CaseStyle::Upper
    } else if word.chars().skip(1).all(|c| !c.is_uppercase()) {
        CaseStyle::Title
    } else {
        CaseStyle::AsIs // mixed case ("McDonald"): don't guess, leave table misses to the rule
    }
}

fn restyle(s: String, style: CaseStyle) -> String {
    match style {
        CaseStyle::AsIs => s,
        CaseStyle::Upper => s.to_uppercase(),
        CaseStyle::Title => English::capitalize_first(&s),
    }
}

/// Run a regular-rule fallback with the same case handling table hits get:
/// Title/ALL-CAPS input is lowercased for the rule and the casing restored on
/// the output — so `HOT` gives `HOTTER` exactly as `GOOD` gives `BEST` from the
/// table, instead of the mixed-case `HOTter` the raw rule would produce. Mixed
/// case runs the rule on the input as-is, unchanged behavior.
fn rule_with_case(word: &str, rule: impl Fn(&str) -> String) -> String {
    match case_style(word) {
        CaseStyle::AsIs => rule(word),
        style => restyle(rule(&word.to_lowercase()), style),
    }
}

/// Lowercase table rows shadowed by a common ALL-CAPS/Title-case homograph:
/// `US`/`Us` (the acronym / its title-casing) must not pluralize through the
/// closed-class row `"us" => "uss"`. The case-folded fallback skips these keys
/// — such input inflects by rule instead — while every other key restyles as
/// usual (`CHILD -> CHILDREN`). Curated and tiny; extend it if another common
/// acronym collides with an attested irregular row.
const ACRONYM_SHADOWED: &[&str] = &["us"];

/// Case-insensitive table lookup: the exact key first; for Title/ALL-CAPS input,
/// fall back to the lowercase key (the tables are all-lowercase) and remember how
/// to restore the casing onto the value. Regular-rule fallbacks never need this —
/// they build on the input string and preserve its casing naturally.
fn ci_lookup<T>(word: &str, get: impl Fn(&str) -> Option<T>) -> Option<(T, CaseStyle)> {
    if let Some(v) = get(word) {
        return Some((v, CaseStyle::AsIs));
    }
    let style = case_style(word);
    if style != CaseStyle::AsIs {
        let lower = word.to_lowercase();
        if !ACRONYM_SHADOWED.contains(&lower.as_str())
            && let Some(v) = get(&lower)
        {
            return Some((v, style));
        }
    }
    None
}

/// `ci_lookup` on the word, then on its base lemma — skipping the second,
/// byte-identical probe when no sense suffix stripped (the common case: most
/// input has no `_<n>` suffix, so `base == word`).
fn ci_lookup_with_base<T>(
    word: &str,
    base: &str,
    get: impl Fn(&str) -> Option<T>,
) -> Option<(T, CaseStyle)> {
    ci_lookup(word, &get).or_else(|| {
        if base != word {
            ci_lookup(base, &get)
        } else {
            None
        }
    })
}

/// Entry point for English inflection and morphology.
///
/// `English` is the low-level `&str` API for handling English nouns, verbs,
/// adjectives, pronouns, and possessives. It delegates irregular forms to
/// lookup tables and falls back on [`EnglishCore`] for regular inflection
/// rules. For noun counting ergonomics, see [`count`] and
/// [`count_with_number`].
pub struct English;
impl English {
    /// Inflects a noun into singular or plural form.
    ///
    /// Handles irregular nouns (e.g., `"child" -> "children"`) and
    /// falls back to regular pluralization rules when no table entry exists.
    /// Sense-numbered keys resolve to their homograph (`"die_2" -> "dice"`);
    /// a `_<n>` suffix that resolves to no table key stays opaque
    /// (`"cat_2" -> "cat_2s"`).
    ///
    /// # Examples
    /// ```rust
    /// use english::{English, Number};
    ///
    /// assert_eq!(English::noun("cat", &Number::Plural), "cats");
    /// assert_eq!(English::noun("child", &Number::Plural), "children");
    /// assert_eq!(English::noun("die_2", &Number::Plural), "dice");
    /// ```
    pub fn noun(word: &str, number: &Number) -> String {
        let base_word = base_lemma(word, |w| ci_lookup(w, get_plural).is_some());

        match number {
            Number::Singular => base_word.to_string(),
            Number::Plural => {
                if let Some((x, style)) = ci_lookup_with_base(word, base_word, get_plural) {
                    restyle(x.to_owned(), style)
                } else {
                    rule_with_case(base_word, |w| EnglishCore::noun(w, number))
                }
            }
        }
    }

    /// Inflects an adjective into positive, comparative, or superlative form.
    ///
    /// Handles irregular adjectives (e.g., `"good" -> "better"/"best"`)
    /// and falls back to regular periphrastic forms
    /// (e.g., `"fun" -> "more fun"/"most fun"`).
    /// Sense-numbered keys resolve to their pinned homograph
    /// (`"bad_2"` -> `"badder"`); a suffix that resolves to no table key stays
    /// opaque.
    ///
    /// # Examples
    /// ```rust
    /// use english::{Degree, English};
    ///
    /// assert_eq!(English::adj("fast", &Degree::Comparative), "faster");
    /// assert_eq!(English::adj("good", &Degree::Superlative), "best");
    /// assert_eq!(English::adj("fun", &Degree::Comparative), "more fun");
    /// ```
    pub fn adj(word: &str, degree: &Degree) -> String {
        let base_word = base_lemma(word, |w| ci_lookup(w, get_adjective_forms).is_some());
        // The table lookup lives inside the arms that use it: Positive returns
        // the base lemma directly and must not pay two probes for nothing.
        match degree {
            Degree::Positive => base_word.to_owned(),
            Degree::Comparative => {
                match ci_lookup_with_base(word, base_word, get_adjective_forms) {
                    Some(((comp, _), style)) => restyle(comp.to_owned(), style),
                    None => rule_with_case(base_word, EnglishCore::comparative),
                }
            }
            Degree::Superlative => {
                match ci_lookup_with_base(word, base_word, get_adjective_forms) {
                    Some(((_, sup), style)) => restyle(sup.to_owned(), style),
                    None => rule_with_case(base_word, EnglishCore::superlative),
                }
            }
        }
    }

    /// Inflects an adverb into positive, comparative, or superlative form.
    ///
    /// The regular rule is periphrastic (`"quickly" -> "more quickly"/"most
    /// quickly"`); the small closed set of flat and suppletive adverbs is served
    /// from the table (`"well" -> "better"/"best"`, `"fast" -> "faster"/"fastest"`,
    /// `"far" -> "farther"/"farthest"`). Sense-numbered keys resolve to their
    /// pinned homograph (`"far_2" -> "further"`); a suffix that resolves to no
    /// table key stays opaque.
    ///
    /// # Examples
    /// ```rust
    /// use english::{Degree, English};
    ///
    /// assert_eq!(English::adverb("quickly", &Degree::Comparative), "more quickly");
    /// assert_eq!(English::adverb("well", &Degree::Comparative), "better");
    /// assert_eq!(English::adverb("fast", &Degree::Superlative), "fastest");
    /// ```
    pub fn adverb(word: &str, degree: &Degree) -> String {
        let base_word = base_lemma(word, |w| ci_lookup(w, get_adverb_forms).is_some());
        // The table lookup lives inside the arms that use it: Positive returns the
        // base lemma directly and must not pay two probes for nothing.
        match degree {
            Degree::Positive => base_word.to_owned(),
            Degree::Comparative => match ci_lookup_with_base(word, base_word, get_adverb_forms) {
                Some(((comp, _), style)) => restyle(comp.to_owned(), style),
                None => rule_with_case(base_word, EnglishCore::comparative_adverb),
            },
            Degree::Superlative => match ci_lookup_with_base(word, base_word, get_adverb_forms) {
                Some(((_, sup), style)) => restyle(sup.to_owned(), style),
                None => rule_with_case(base_word, EnglishCore::superlative_adverb),
            },
        }
    }

    /// Conjugates a verb into the requested form.
    ///
    /// Handles irregular verbs (e.g., `"go" -> "went"`, `"eat" -> "ate"`)
    /// and falls back to regular conjugation rules when no table entry exists.
    /// Supports sense-numbered keys for homographs (`"lie"` -> `"lay"` for the
    /// recline sense, `"lie_2"` -> `"lied"` for the untruth sense, in the past
    /// finite).
    ///
    /// # Examples
    /// ```rust
    /// use english::{English, Form, Number, Person, Tense};
    ///
    /// // Regular verb
    /// assert_eq!(
    ///     English::verb("walk", &Person::Third, &Number::Singular, &Tense::Present, &Form::Finite),
    ///     "walks"
    /// );
    ///
    /// // Irregular verb
    /// assert_eq!(
    ///     English::verb("eat", &Person::Third, &Number::Singular, &Tense::Past, &Form::Finite),
    ///     "ate"
    /// );
    ///
    /// // Participle
    /// assert_eq!(
    ///     English::verb("go", &Person::Third, &Number::Plural, &Tense::Past, &Form::Participle),
    ///     "gone"
    /// );
    ///
    /// // Homograph senses (numbered by form signature: recline < untruth)
    /// assert_eq!(
    ///     English::verb("lie", &Person::Third, &Number::Singular, &Tense::Past, &Form::Finite),
    ///     "lay"
    /// );
    /// assert_eq!(
    ///     English::verb("lie_2", &Person::Third, &Number::Singular, &Tense::Past, &Form::Finite),
    ///     "lied"
    /// );
    /// ```
    pub fn verb(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
    ) -> String {
        let base_word = base_lemma(word, |w| ci_lookup(w, get_verb_forms).is_some());
        match ci_lookup_with_base(word, base_word, get_verb_forms) {
            Some((wordik, style)) => match (person, number, tense, form) {
                // Branches that return the lemma itself keep the caller's own
                // casing; branches that return a table value restore it.
                (_, _, _, Form::Infinitive) => base_word.to_owned(),
                (Person::Third, Number::Singular, Tense::Present, Form::Finite) => {
                    restyle(wordik.0.to_string(), style)
                }
                (_, _, Tense::Present, Form::Finite) => base_word.to_owned(),
                (_, _, Tense::Present, Form::Participle) => restyle(wordik.2.to_owned(), style),
                (_, _, Tense::Past, Form::Participle) => restyle(wordik.3.to_owned(), style),
                (_, _, Tense::Past, Form::Finite) => restyle(wordik.1.to_owned(), style),
            },
            None => rule_with_case(base_word, |w| {
                EnglishCore::verb(w, person, number, tense, form)
            }),
        }
    }
    /// Returns the correct English pronoun for the given grammatical features.
    ///
    /// # Examples
    /// ```rust
    /// use english::{Case, English, Gender, Number, Person};
    ///
    /// assert_eq!(
    ///     English::pronoun(&Person::First, &Number::Singular, &Gender::Neuter, &Case::Nominative),
    ///     "I"
    /// );
    /// assert_eq!(
    ///     English::pronoun(&Person::Third, &Number::Singular, &Gender::Feminine, &Case::Nominative),
    ///     "she"
    /// );
    /// assert_eq!(
    ///     English::pronoun(&Person::Third, &Number::Plural, &Gender::Neuter, &Case::Nominative),
    ///     "they"
    /// );
    /// ```
    pub fn pronoun(person: &Person, number: &Number, gender: &Gender, case: &Case) -> &'static str {
        EnglishCore::pronoun(person, number, gender, case)
    }
    /// Adds an English possessive suffix (`'s` or `'`) to a word.
    ///
    /// Current policy is simple and style-neutral: any word ending in `s` gets a
    /// bare apostrophe (`dogs'`, but also `bus'`/`James'`), and everything else
    /// gets `'s`. Callers that need a style guide's singular-`s` preference
    /// (`bus's`, `James's`) should handle that policy themselves for now.
    ///
    /// # Examples
    /// ```rust
    /// use english::English;
    ///
    /// assert_eq!(English::add_possessive("dog"), "dog's");
    /// assert_eq!(English::add_possessive("dogs"), "dogs'");
    /// ```
    pub fn add_possessive(word: &str) -> String {
        EnglishCore::add_possessive(word)
    }

    /// Capitalizes the first letter of a string.
    ///
    /// # Examples
    /// ```rust
    /// use english::English;
    ///
    /// assert_eq!(English::capitalize_first(""), "");
    /// assert_eq!(English::capitalize_first("house"), "House");
    /// ```
    pub fn capitalize_first(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}

/// Inflect a noun according to a numeric count.
///
/// This helper is intentionally tiny: it accepts only `u32`, treats exactly `1`
/// as singular, and treats every other value (`0`, `2`, ...) as plural. Decimal,
/// negative, localized, or formatted counts should be handled by callers.
///
/// # Examples
/// ```rust
/// use english::count;
///
/// assert_eq!(count("cat", 1), "cat");
/// assert_eq!(count("cat", 2), "cats");
/// ```
pub fn count(noun: &str, count: u32) -> String {
    if count == 1 {
        English::noun(noun, &Number::Singular)
    } else {
        English::noun(noun, &Number::Plural)
    }
}

/// Inflect a noun according to a numeric count and keep the number in the
/// output.
///
/// This is a display convenience over [`count`]; it does not handle decimal,
/// negative, localized, or custom-formatted quantities.
///
/// # Examples
/// ```rust
/// use english::count_with_number;
///
/// assert_eq!(count_with_number("cat", 1), "1 cat");
/// assert_eq!(count_with_number("cat", 2), "2 cats");
/// ```
pub fn count_with_number(noun: &str, amount: u32) -> String {
    format!("{} {}", amount, count(noun, amount))
}

#[cfg(test)]
mod rule_table_sync_tests {
    //! Guard on the layering contract between `english-core`'s regular rules and the
    //! generated tables: a BARE key's row must hold something the rules cannot
    //! predict — a bare row equal to the rule's output is dead weight, and its
    //! appearance means a core rule changed without regenerating the tables.

    use super::{ADJECTIVE_MAP, ADVERB_MAP, PLURAL_MAP, VERB_MAP, canonical_sense_suffix_base};
    use english_core::EnglishCore;
    use english_core::grammar::{Form, Number, Person, Tense};

    fn is_bare(key: &str) -> bool {
        canonical_sense_suffix_base(key).is_none()
    }

    #[test]
    fn bare_rows_hold_only_what_the_rules_cannot_predict() {
        // Every BARE table row must differ from the rule prediction — the
        // generator drops rule-equal candidates, so a bare row equal to the rule
        // is dead weight meaning a core rule changed without regenerating. No
        // homograph-base exemption: a base like `lie` is a genuine irregular, so
        // checking it too only strengthens the guard (matches `audit_tables`).
        let nouns: Vec<_> = PLURAL_MAP.entries().map(|(k, v)| (*k, *v)).collect();
        let verbs: Vec<_> = VERB_MAP.entries().map(|(k, v)| (*k, *v)).collect();
        let adjs: Vec<_> = ADJECTIVE_MAP.entries().map(|(k, v)| (*k, *v)).collect();
        let adverbs: Vec<_> = ADVERB_MAP.entries().map(|(k, v)| (*k, *v)).collect();
        let mut redundant = Vec::new();

        for (key, plural) in &nouns {
            if is_bare(key) && EnglishCore::pluralize_noun(key) == *plural {
                redundant.push(format!("noun {key} -> {plural} (rule already predicts it)"));
            }
        }

        for (key, (third, past, pres_part, past_part)) in &verbs {
            if !is_bare(key) {
                continue;
            }
            let p = |tense: &Tense, form: &Form| {
                EnglishCore::verb(key, &Person::Third, &Number::Singular, tense, form)
            };
            if p(&Tense::Present, &Form::Finite) == *third
                && p(&Tense::Past, &Form::Finite) == *past
                && p(&Tense::Present, &Form::Participle) == *pres_part
                && p(&Tense::Past, &Form::Participle) == *past_part
            {
                redundant.push(format!("verb {key} (rule already predicts all four forms)"));
            }
        }

        for (key, (comp, sup)) in &adjs {
            if is_bare(key)
                && EnglishCore::comparative(key) == *comp
                && EnglishCore::superlative(key) == *sup
            {
                redundant.push(format!(
                    "adj {key} -> {comp}/{sup} (rule already predicts it)"
                ));
            }
        }

        for (key, (comp, sup)) in &adverbs {
            if is_bare(key)
                && EnglishCore::comparative_adverb(key) == *comp
                && EnglishCore::superlative_adverb(key) == *sup
            {
                redundant.push(format!(
                    "adverb {key} -> {comp}/{sup} (rule already predicts it)"
                ));
            }
        }

        assert!(
            redundant.is_empty(),
            "{} table row(s) are redundant with the regular rules — a core rule changed \
             without regenerating: run `cargo xtask refresh-data`:\n  {}",
            redundant.len(),
            redundant.join("\n  ")
        );
    }
}
