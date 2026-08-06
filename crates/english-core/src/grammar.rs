//! The grammatical feature enums that parameterize every inflection query.
//!
//! These are deliberately minimal — only distinctions English morphology
//! actually realizes are present (no Future tense variant: English futures are
//! periphrastic "will + infinitive", the caller's composition job). All enums
//! are `Copy` and passed by reference for historical API-stability reasons.

/// Grammatical number.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Number {
    /// One entity.
    Singular,
    /// More than one entity, or any count other than exactly one in the small
    /// counting helpers.
    Plural,
}

/// Pronoun case.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Case {
    /// Subject form: `I`, `he`, `they`.
    Nominative,
    /// Object form: `me`, `him`, `them`.
    Accusative,
    /// Reflexive form: `myself`, `himself`, `themselves`.
    Reflexive,
    /// Independent possessive pronoun: `mine`, `his`, `theirs`.
    Possessive,
    /// Possessive determiner: `my`, `your`, `their`.
    PersonalPossessive,
}

/// Morphological tense realized by finite/simple verb forms.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tense {
    /// Present tense (`walk`, `walks`, `is`).
    Present,
    /// Past tense (`walked`, `went`, `was`).
    Past,
    // Future is periphrastic in English (`will` + infinitive), so callers compose it.
}

/// Verb form requested from the conjugator.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Form {
    /// Finite form agreeing with person/number when English realizes agreement.
    Finite,
    /// Participle form. Combined with [`Tense::Present`] for `-ing`, or
    /// [`Tense::Past`] for past participles.
    Participle,
    /// Bare infinitive lemma.
    Infinitive,
    // Transgressive, Supine, etc., depending on language
}

/// Grammatical person.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Person {
    First,
    Second,
    Third,
}

/// Pronoun gender for third-person singular forms.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

/// Degree for adjectives and gradable adverbs.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Degree {
    Positive,
    Comparative,
    Superlative,
}

/// Animacy marker reserved for higher-level phrase APIs.
///
/// The current `english` and `english-core` inflection functions do not consume
/// this enum; it remains exported for source compatibility with earlier grammar
/// experiments.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Animacy {
    Animate,
    Inanimate,
}

/*#[derive(Debug, PartialEq, Clone)]
pub enum Mood {
    Indicative,
    Subjunctive,
    Imperative,
    // Conditional, Interrogative, etc.
} */
/*#[derive(Debug, PartialEq, Clone)]
pub enum Det {
    Definite,
    Indefinite,
}
 */
/*#[derive(Debug, PartialEq, Clone)]
pub enum Voice {
    Active,
    Passive,
    // Middle, Reflexive, etc.
}
*/
