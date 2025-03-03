pub struct English {}
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
    SimplePresent,
    SimplePast,
    ParticiplePast,
    ParticiplePresent,
    SubjunctivePresent,
    SubjunctivePast,
    ImperativePresent,
    //ImperativePast,
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
