pub struct English {}

#[derive(Debug, PartialEq, Clone)]
pub struct VerbFormSpec {
    pub tense: Option<Tense>,
    pub mood: Option<Mood>,
    pub form: VerbForm,
    pub person: Person,
    pub number: Number,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NounFormSpec {
    pub number: Number,
    pub case: Case,
    pub gender: Option<Gender>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Singular,
    Plural,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Case {
    Nominative,
    Accusative,
    Reflexive,
    Possessive,
    PersonalPossesive,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Tense {
    Present,
    Past,
    // Future could be added too
}

#[derive(Debug, PartialEq, Clone)]
pub enum Mood {
    Indicative,
    Subjunctive,
    Imperative,
    // Conditional, Interrogative, etc.
}

#[derive(Debug, PartialEq, Clone)]
pub enum VerbForm {
    Finite,
    Participle,
    Infinitive,
    Gerund,
    // Transgressive, Supine, etc., depending on language
}

#[derive(Debug, PartialEq, Clone)]
pub enum Voice {
    Active,
    Passive,
    // Middle, Reflexive, etc.
}

#[derive(Debug, PartialEq, Clone)]
pub enum Person {
    First,
    Second,
    Third,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}
