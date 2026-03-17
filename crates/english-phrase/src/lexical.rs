use english::{Adj, Case, English, Gender, Noun, Number, Person, Verb};

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
pub struct Pronoun {
    person: Person,
    number: Number,
    gender: Gender,
}

#[allow(non_upper_case_globals)]
impl Pronoun {
    pub const I: Self = Self::new(Person::First, Number::Singular, Gender::Neuter);
    pub const You: Self = Self::new(Person::Second, Number::Singular, Gender::Neuter);
    pub const He: Self = Self::new(Person::Third, Number::Singular, Gender::Masculine);
    pub const She: Self = Self::new(Person::Third, Number::Singular, Gender::Feminine);
    pub const It: Self = Self::new(Person::Third, Number::Singular, Gender::Neuter);
    pub const We: Self = Self::new(Person::First, Number::Plural, Gender::Neuter);
    pub const They: Self = Self::new(Person::Third, Number::Plural, Gender::Neuter);
    pub const YouPlural: Self = Self::new(Person::Second, Number::Plural, Gender::Neuter);

    pub const fn new(person: Person, number: Number, gender: Gender) -> Self {
        Self {
            person,
            number,
            gender,
        }
    }

    fn form(self, case: Case) -> &'static str {
        English::pronoun(&self.person, &self.number, &self.gender, &case)
    }

    pub fn subject_form(self) -> &'static str {
        self.form(Case::Nominative)
    }

    pub fn object_form(self) -> &'static str {
        self.form(Case::Accusative)
    }

    pub fn possessive_dependent_form(self) -> &'static str {
        self.form(Case::PersonalPossesive)
    }

    pub fn possessive_independent_form(self) -> &'static str {
        self.form(Case::Possessive)
    }

    pub fn reflexive_form(self) -> &'static str {
        self.form(Case::Reflexive)
    }

    pub fn as_str(self) -> &'static str {
        self.subject_form()
    }

    pub const fn person(self) -> Person {
        self.person
    }

    pub const fn number(self) -> Number {
        self.number
    }

    pub const fn gender(self) -> Gender {
        self.gender
    }

    pub const fn second_plural() -> Self {
        Self::YouPlural
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
