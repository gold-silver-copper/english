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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Animacy {
    Animate,
    Inanimate,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Pronoun {
    I,
    Me,
    You,
    He,
    Him,
    She,
    Her,
    It,
    We,
    Us,
    They,
    Them,
}

impl Pronoun {
    pub fn person(self) -> Person {
        match self {
            Self::I | Self::Me | Self::We | Self::Us => Person::First,
            Self::You => Person::Second,
            Self::He | Self::Him | Self::She | Self::Her | Self::It | Self::They | Self::Them => {
                Person::Third
            }
        }
    }

    pub fn number(self) -> Number {
        match self {
            Self::We | Self::Us | Self::They | Self::Them => Number::Plural,
            _ => Number::Singular,
        }
    }

    pub fn gender(self) -> Gender {
        match self {
            Self::He | Self::Him => Gender::Masculine,
            Self::She | Self::Her => Gender::Feminine,
            _ => Gender::Neuter,
        }
    }

    pub fn animacy(self) -> Animacy {
        match self {
            Self::It => Animacy::Inanimate,
            _ => Animacy::Animate,
        }
    }

    pub fn canonical_case(self) -> Case {
        match self {
            Self::Me | Self::Him | Self::Her | Self::Us | Self::Them => Case::Accusative,
            _ => Case::Nominative,
        }
    }

    pub fn agreement(self) -> (Person, Number, Gender) {
        (self.person(), self.number(), self.gender())
    }
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
