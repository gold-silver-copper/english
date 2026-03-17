#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Number {
    Singular,
    Plural,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Case {
    Nominative,
    Accusative,
    Reflexive,
    Possessive,
    PersonalPossesive,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tense {
    Present,
    Past,
    // Future could be added too
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Form {
    Finite,
    Participle,
    Infinitive,
    // Transgressive, Supine, etc., depending on language
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Person {
    First,
    Second,
    Third,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Degree {
    Positive,
    Comparative,
    Superlative,
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
