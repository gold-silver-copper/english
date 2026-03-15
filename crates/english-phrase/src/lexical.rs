use english::{Adj, Noun, Number, Person, Verb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Determiner {
    The,
    Indefinite,
    This,
    That,
    These,
    Those,
}

impl Determiner {
    pub fn as_str(self) -> &'static str {
        match self {
            Determiner::The => "the",
            Determiner::Indefinite => "a",
            Determiner::This => "this",
            Determiner::That => "that",
            Determiner::These => "these",
            Determiner::Those => "those",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pronoun {
    I,
    You,
    He,
    She,
    It,
    We,
    They,
}

impl Pronoun {
    pub fn subject_form(self) -> &'static str {
        match self {
            Pronoun::I => "i",
            Pronoun::You => "you",
            Pronoun::He => "he",
            Pronoun::She => "she",
            Pronoun::It => "it",
            Pronoun::We => "we",
            Pronoun::They => "they",
        }
    }

    pub fn object_form(self) -> &'static str {
        match self {
            Pronoun::I => "me",
            Pronoun::You => "you",
            Pronoun::He => "him",
            Pronoun::She => "her",
            Pronoun::It => "it",
            Pronoun::We => "us",
            Pronoun::They => "them",
        }
    }

    pub fn possessive_dependent_form(self) -> &'static str {
        match self {
            Pronoun::I => "my",
            Pronoun::You => "your",
            Pronoun::He => "his",
            Pronoun::She => "her",
            Pronoun::It => "its",
            Pronoun::We => "our",
            Pronoun::They => "their",
        }
    }

    pub fn reflexive_form(self) -> &'static str {
        match self {
            Pronoun::I => "myself",
            Pronoun::You => "yourself",
            Pronoun::He => "himself",
            Pronoun::She => "herself",
            Pronoun::It => "itself",
            Pronoun::We => "ourselves",
            Pronoun::They => "themselves",
        }
    }

    pub fn as_str(self) -> &'static str {
        self.subject_form()
    }

    pub fn person(self) -> Person {
        match self {
            Pronoun::I | Pronoun::We => Person::First,
            Pronoun::You => Person::Second,
            Pronoun::He | Pronoun::She | Pronoun::It | Pronoun::They => Person::Third,
        }
    }

    pub fn number(self) -> Number {
        match self {
            Pronoun::We | Pronoun::They => Number::Plural,
            Pronoun::I | Pronoun::You | Pronoun::He | Pronoun::She | Pronoun::It => {
                Number::Singular
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounEntry {
    lemma: Noun,
}

impl NounEntry {
    pub fn new(lemma: impl Into<Noun>) -> Self {
        Self {
            lemma: lemma.into(),
        }
    }

    pub fn lemma(&self) -> &Noun {
        &self.lemma
    }

    pub fn as_str(&self) -> &str {
        self.lemma.as_str()
    }
}

impl From<&str> for NounEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for NounEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbEntry {
    lemma: Verb,
}

impl VerbEntry {
    pub fn new(lemma: impl Into<Verb>) -> Self {
        Self {
            lemma: lemma.into(),
        }
    }

    pub fn lemma(&self) -> &Verb {
        &self.lemma
    }

    pub fn as_str(&self) -> &str {
        self.lemma.as_str()
    }
}

impl From<&str> for VerbEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for VerbEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjectiveEntry {
    lemma: Adj,
}

impl AdjectiveEntry {
    pub fn new(lemma: impl Into<Adj>) -> Self {
        Self {
            lemma: lemma.into(),
        }
    }

    pub fn lemma(&self) -> &Adj {
        &self.lemma
    }

    pub fn as_str(&self) -> &str {
        self.lemma.as_str()
    }
}

impl From<&str> for AdjectiveEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AdjectiveEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdverbEntry {
    text: String,
}

impl AdverbEntry {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl From<&str> for AdverbEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AdverbEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepositionEntry {
    text: String,
}

impl PrepositionEntry {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl From<&str> for PrepositionEntry {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for PrepositionEntry {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
