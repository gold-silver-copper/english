use std::ops::Add;

use english::{Animacy, Gender, Number, Person};

use crate::cat::{bwd, fwd, Cat};
use crate::derivation::{token, AgreementInfo, Ccg, TokenKind};
use crate::lexicon::LexEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    Can,
    Must,
    Should,
    Will,
    Would,
    Have,
    Be,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conj {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrepRole {
    Complement,
    Adverbial,
    Adnominal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerbFormKind {
    Past,
    Present,
    Bare,
    Perfective,
    Progressive,
    PastParticipleAdj,
    PresentParticipleAdj,
    Passive,
    PassiveBy,
}

#[derive(Debug, Clone)]
pub struct VerbBuilder {
    lemma: String,
    cat: Cat,
}

#[derive(Debug, Clone)]
pub struct PrepBuilder {
    lemma: String,
}

impl Pronoun {
    pub(crate) fn agreement(self) -> AgreementInfo {
        match self {
            Self::I | Self::Me => AgreementInfo {
                person: Person::First,
                number: Number::Singular,
                gender: Gender::Neuter,
            },
            Self::You => AgreementInfo {
                person: Person::Second,
                number: Number::Singular,
                gender: Gender::Neuter,
            },
            Self::He | Self::Him => AgreementInfo {
                person: Person::Third,
                number: Number::Singular,
                gender: Gender::Masculine,
            },
            Self::She | Self::Her => AgreementInfo {
                person: Person::Third,
                number: Number::Singular,
                gender: Gender::Feminine,
            },
            Self::It => AgreementInfo {
                person: Person::Third,
                number: Number::Singular,
                gender: Gender::Neuter,
            },
            Self::We | Self::Us => AgreementInfo {
                person: Person::First,
                number: Number::Plural,
                gender: Gender::Neuter,
            },
            Self::They | Self::Them => AgreementInfo {
                person: Person::Third,
                number: Number::Plural,
                gender: Gender::Neuter,
            },
        }
    }

    pub(crate) fn animacy(self) -> Animacy {
        match self {
            Self::It => Animacy::Inanimate,
            _ => Animacy::Animate,
        }
    }
}

impl Modal {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Can => "can",
            Self::Must => "must",
            Self::Should => "should",
            Self::Will => "will",
            Self::Would => "would",
            Self::Have => "have",
            Self::Be => "be",
        }
    }
}

impl Conj {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

impl VerbBuilder {
    pub fn past(self) -> Ccg {
        let cat = self.cat.clone();
        self.build(VerbFormKind::Past, cat)
    }

    pub fn present(self) -> Ccg {
        let cat = self.cat.clone();
        self.build(VerbFormKind::Present, cat)
    }

    pub fn bare(self) -> Ccg {
        let cat = self.cat.clone();
        self.build(VerbFormKind::Bare, cat)
    }

    pub fn perfective(self) -> Ccg {
        let cat = self.cat.clone();
        self.build(VerbFormKind::Perfective, cat)
    }

    pub fn progressive(self) -> Ccg {
        let cat = self.cat.clone();
        self.build(VerbFormKind::Progressive, cat)
    }

    pub fn past_participle(self) -> Ccg {
        self.build(VerbFormKind::PastParticipleAdj, fwd(Cat::N, Cat::N))
    }

    pub fn present_participle(self) -> Ccg {
        self.build(VerbFormKind::PresentParticipleAdj, fwd(Cat::N, Cat::N))
    }

    pub fn passive(self) -> Ccg {
        self.build(VerbFormKind::Passive, bwd(Cat::S, Cat::NP))
    }

    pub fn passive_by(self) -> Ccg {
        self.build(VerbFormKind::PassiveBy, fwd(bwd(Cat::S, Cat::NP), Cat::NP))
    }

    fn build(self, form: VerbFormKind, cat: Cat) -> Ccg {
        token(
            self.lemma.clone(),
            cat,
            TokenKind::Verb {
                lemma: self.lemma,
                form,
            },
            None,
            None,
        )
    }
}

impl PrepBuilder {
    pub fn complement(self) -> Ccg {
        token(
            self.lemma.clone(),
            fwd(Cat::PP, Cat::NP),
            TokenKind::Prep {
                _lemma: self.lemma,
                _role: PrepRole::Complement,
            },
            None,
            None,
        )
    }

    pub fn adverbial(self) -> Ccg {
        token(
            self.lemma.clone(),
            fwd(bwd(bwd(Cat::S, Cat::NP), bwd(Cat::S, Cat::NP)), Cat::NP),
            TokenKind::Prep {
                _lemma: self.lemma,
                _role: PrepRole::Adverbial,
            },
            None,
            None,
        )
    }

    pub fn adnominal(self) -> Ccg {
        token(
            self.lemma.clone(),
            fwd(bwd(Cat::N, Cat::N), Cat::NP),
            TokenKind::Prep {
                _lemma: self.lemma,
                _role: PrepRole::Adnominal,
            },
            None,
            None,
        )
    }
}

impl From<PrepBuilder> for Ccg {
    fn from(value: PrepBuilder) -> Self {
        value.complement()
    }
}

impl Add<Ccg> for PrepBuilder {
    type Output = Ccg;

    fn add(self, rhs: Ccg) -> Self::Output {
        self.complement() + rhs
    }
}

impl Add<PrepBuilder> for Ccg {
    type Output = Ccg;

    fn add(self, rhs: PrepBuilder) -> Self::Output {
        self + rhs.complement()
    }
}

pub fn name(entry: LexEntry) -> Ccg {
    let (surface, cat, animacy, agreement) = entry.into_parts();
    token(
        surface,
        cat,
        TokenKind::Name,
        animacy.or(Some(Animacy::Animate)),
        agreement.or(Some(AgreementInfo {
            person: Person::Third,
            number: Number::Singular,
            gender: Gender::Neuter,
        })),
    )
}

pub fn pronoun(p: Pronoun) -> Ccg {
    token(
        match p {
            Pronoun::I | Pronoun::Me => "I",
            Pronoun::You => "you",
            Pronoun::He | Pronoun::Him => "he",
            Pronoun::She | Pronoun::Her => "she",
            Pronoun::It => "it",
            Pronoun::We | Pronoun::Us => "we",
            Pronoun::They | Pronoun::Them => "they",
        },
        Cat::NP,
        TokenKind::Pronoun { pronoun: p },
        Some(p.animacy()),
        Some(p.agreement()),
    )
}

pub fn noun(entry: LexEntry) -> Ccg {
    let (surface, cat, animacy, agreement) = entry.into_parts();
    token(
        surface.clone(),
        cat,
        TokenKind::Noun { lemma: surface },
        animacy,
        agreement.or(Some(AgreementInfo {
            person: Person::Third,
            number: Number::Singular,
            gender: Gender::Neuter,
        })),
    )
}

pub fn det(s: &str) -> Ccg {
    token(s, fwd(Cat::NP, Cat::N), TokenKind::Plain, None, None)
}

pub fn rel(s: &str) -> Ccg {
    token(
        s,
        fwd(bwd(Cat::N, Cat::N), fwd(Cat::S, Cat::NP)),
        TokenKind::Plain,
        None,
        None,
    )
}

pub fn verb(entry: LexEntry) -> VerbBuilder {
    let (surface, cat, _, _) = entry.into_parts();
    VerbBuilder {
        lemma: surface,
        cat,
    }
}

pub fn modal(m: Modal) -> Ccg {
    token(
        m.as_str(),
        fwd(bwd(Cat::S, Cat::NP), bwd(Cat::S, Cat::NP)),
        TokenKind::Modal { modal: m },
        None,
        None,
    )
}

pub fn inf() -> Ccg {
    token(
        "to",
        fwd(bwd(Cat::S, Cat::NP), bwd(Cat::S, Cat::NP)),
        TokenKind::Plain,
        None,
        None,
    )
}

pub fn adj(s: &str) -> Ccg {
    token(s, fwd(Cat::N, Cat::N), TokenKind::Plain, None, None)
}

pub fn adv(s: &str) -> Ccg {
    token(
        s,
        bwd(bwd(Cat::S, Cat::NP), bwd(Cat::S, Cat::NP)),
        TokenKind::Plain,
        None,
        None,
    )
}

pub fn prep(s: &str) -> PrepBuilder {
    PrepBuilder {
        lemma: s.to_string(),
    }
}

pub fn comp(s: &str) -> Ccg {
    token(s, fwd(Cat::S, Cat::S), TokenKind::Plain, None, None)
}

pub fn gap(cat: Cat) -> Ccg {
    token(
        "",
        fwd(cat.clone(), cat.clone()),
        TokenKind::Gap { original: cat },
        None,
        None,
    )
}
