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

/// The base of a sense-suffixed key: sense suffixes are `_<digits>` (underscore
/// plus a number, e.g. `die_2`). Returns `None` when the word carries no suffix.
///
/// The underscore makes this unambiguous: the extractor never emits a lemma
/// containing `_` (such words fail its character filter), so a trailing
/// `_<digits>` can ONLY be a sense suffix. Digit-bearing words (`"mp3"`, `"F16"`,
/// `"F2"`, `"co2"`) have no underscore and are therefore never mangled — no
/// plausibility heuristics needed.
fn split_sense_suffix(word: &str) -> Option<&str> {
    let (base, digits) = word.rsplit_once('_')?;
    if base.is_empty() || digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(base)
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
fn base_lemma<'a>(word: &'a str, is_key: impl Fn(&str) -> bool) -> &'a str {
    match split_sense_suffix(word) {
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

/// Case-insensitive table lookup: the exact key first; for Title/ALL-CAPS input,
/// fall back to the lowercase key (the tables are all-lowercase) and remember how
/// to restore the casing onto the value. Regular-rule fallbacks never need this —
/// they build on the input string and preserve its casing naturally.
fn ci_lookup<T>(word: &str, get: impl Fn(&str) -> Option<T>) -> Option<(T, CaseStyle)> {
    if let Some(v) = get(word) {
        return Some((v, CaseStyle::AsIs));
    }
    let style = case_style(word);
    if style != CaseStyle::AsIs
        && let Some(v) = get(&word.to_lowercase())
    {
        return Some((v, style));
    }
    None
}

/// Test-only visibility into the generated tables (integration tests cannot see
/// the private generated modules). Not part of the public API.
#[doc(hidden)]
pub fn __table_entries() -> (
    Vec<(&'static str, &'static str)>,
    Vec<(&'static str, (&'static str, &'static str, &'static str, &'static str))>,
    Vec<(&'static str, (&'static str, &'static str))>,
) {
    (
        PLURAL_MAP.entries().map(|(k, v)| (*k, *v)).collect(),
        VERB_MAP.entries().map(|(k, v)| (*k, *v)).collect(),
        ADJECTIVE_MAP.entries().map(|(k, v)| (*k, *v)).collect(),
    )
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
    /// falls back to regular pluralization rules when no override is found.
    /// Strips trailing numbers used for sense disambiguation (`"die_2" -> "dice"`).
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
                if let Some((x, style)) =
                    ci_lookup(word, get_plural).or_else(|| ci_lookup(base_word, get_plural))
                {
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
    /// Strips trailing numbers used for disambiguation (`"bad_2"` -> `"badder"`).
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
        let forms = ci_lookup(word, get_adjective_forms)
            .or_else(|| ci_lookup(base_word, get_adjective_forms));
        match degree {
            Degree::Positive => base_word.to_owned(),
            Degree::Comparative => {
                if let Some(((comp, _), style)) = forms {
                    restyle(comp.to_owned(), style)
                } else {
                    rule_with_case(base_word, EnglishCore::comparative)
                }
            }
            Degree::Superlative => {
                if let Some(((_, sup), style)) = forms {
                    restyle(sup.to_owned(), style)
                } else {
                    rule_with_case(base_word, EnglishCore::superlative)
                }
            }
        }
    }

    /// Conjugates a verb into the requested form.
    ///
    /// Handles irregular verbs (e.g., `"go" -> "went"`, `"eat" -> "ate"`)
    /// and falls back to regular conjugation rules when no override is found.
    /// Supports sense-numbered keys for homographs (`"lie"` -> `"lied"`,
    /// `"lie_2"` -> `"lay"` in the past finite).
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
    /// // Homograph senses
    /// assert_eq!(
    ///     English::verb("lie_2", &Person::Third, &Number::Singular, &Tense::Past, &Form::Finite),
    ///     "lay"
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
        match ci_lookup(word, get_verb_forms).or_else(|| ci_lookup(base_word, get_verb_forms)) {
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
            None => {
                rule_with_case(base_word, |w| EnglishCore::verb(w, person, number, tense, form))
            }
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
