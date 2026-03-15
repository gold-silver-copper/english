use english::{Adj, Noun, Number, Person, Verb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Determiner {
    The,
    A,
    An,
    This,
    That,
    These,
    Those,
}

impl Determiner {
    pub fn as_str(self) -> &'static str {
        match self {
            Determiner::The => "the",
            Determiner::A => "a",
            Determiner::An => "an",
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
    pub fn as_str(self) -> &'static str {
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
