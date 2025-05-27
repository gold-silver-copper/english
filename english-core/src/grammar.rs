pub struct EnglishCore {}
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
pub struct Noun {
    pub word: String,
    pub number: Number,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Verb {
    pub word: String,
    pub person: Person,
    pub tense: Tense,
    pub form: Form,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Form {
    Finite,
    Participle,
    Infinitive,
    // Transgressive, Supine, etc., depending on language
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
