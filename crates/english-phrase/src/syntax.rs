use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
use english::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tense {
    #[default]
    Present,
    Past,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerbForm {
    Finite(Tense),
    #[default]
    BareInfinitive,
    ToInfinitive,
    GerundParticiple,
    PastParticiple,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Phrase {
    DP(Box<DeterminerPhrase>),
    VP(Box<VerbPhrase>),
    PP(Box<PrepositionalPhrase>),
    AdjP(Box<AdjectivePhrase>),
    AdvP(Box<AdverbPhrase>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeterminerHead {
    CommonNoun(NounEntry),
    ProperName(String),
    Pronoun(Pronoun),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeterminerPhrase {
    pub determiner: Option<Determiner>,
    pub head: DeterminerHead,
    pub number: Number,
    pub modifiers: Vec<Box<Phrase>>,
    pub complements: Vec<Box<Phrase>>,
}

impl DeterminerPhrase {
    pub fn common(head: impl Into<NounEntry>) -> Self {
        Self {
            determiner: None,
            head: DeterminerHead::CommonNoun(head.into()),
            number: Number::Singular,
            modifiers: Vec::new(),
            complements: Vec::new(),
        }
    }

    pub fn proper_name(name: impl Into<String>) -> Self {
        Self {
            determiner: None,
            head: DeterminerHead::ProperName(name.into()),
            number: Number::Singular,
            modifiers: Vec::new(),
            complements: Vec::new(),
        }
    }

    pub fn pronoun(pronoun: Pronoun) -> Self {
        Self {
            determiner: None,
            head: DeterminerHead::Pronoun(pronoun),
            number: pronoun.number(),
            modifiers: Vec::new(),
            complements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjectivePhrase {
    pub modifier: Option<Box<Phrase>>,
    pub head: AdjectiveEntry,
    pub complements: Vec<Box<Phrase>>,
}

impl AdjectivePhrase {
    pub fn new(head: impl Into<AdjectiveEntry>) -> Self {
        Self {
            modifier: None,
            head: head.into(),
            complements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdverbPhrase {
    pub modifier: Option<Box<Phrase>>,
    pub head: AdverbEntry,
    pub complements: Vec<Box<Phrase>>,
}

impl AdverbPhrase {
    pub fn new(head: impl Into<AdverbEntry>) -> Self {
        Self {
            modifier: None,
            head: head.into(),
            complements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrepositionalPhrase {
    pub head: PrepositionEntry,
    pub complement: Box<Phrase>,
}

impl PrepositionalPhrase {
    pub fn new(head: impl Into<PrepositionEntry>, complement: impl Into<Phrase>) -> Self {
        Self {
            head: head.into(),
            complement: Box::new(complement.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase {
    pub head: VerbEntry,
    pub form: VerbForm,
    pub negative: bool,
    pub complements: Vec<Box<Phrase>>,
    pub adjuncts: Vec<Box<Phrase>>,
}

impl VerbPhrase {
    pub fn new(head: impl Into<VerbEntry>) -> Self {
        Self {
            head: head.into(),
            form: VerbForm::BareInfinitive,
            negative: false,
            complements: Vec::new(),
            adjuncts: Vec::new(),
        }
    }

    pub fn finite(head: impl Into<VerbEntry>, tense: Tense) -> Self {
        Self {
            form: VerbForm::Finite(tense),
            ..Self::new(head)
        }
    }
}

pub fn dp(head: impl Into<NounEntry>) -> DeterminerPhrase {
    DeterminerPhrase::common(head)
}

pub fn proper_name(name: impl Into<String>) -> DeterminerPhrase {
    DeterminerPhrase::proper_name(name)
}

pub fn pronoun_dp(pronoun: Pronoun) -> DeterminerPhrase {
    DeterminerPhrase::pronoun(pronoun)
}

pub fn adjp(head: impl Into<AdjectiveEntry>) -> AdjectivePhrase {
    AdjectivePhrase::new(head)
}

pub fn advp(head: impl Into<AdverbEntry>) -> AdverbPhrase {
    AdverbPhrase::new(head)
}

pub fn pp(head: impl Into<PrepositionEntry>, complement: impl Into<Phrase>) -> PrepositionalPhrase {
    PrepositionalPhrase::new(head, complement)
}

pub fn vp(head: impl Into<VerbEntry>) -> VerbPhrase {
    VerbPhrase::new(head)
}

impl From<DeterminerPhrase> for Phrase {
    fn from(value: DeterminerPhrase) -> Self {
        Phrase::DP(Box::new(value))
    }
}

impl From<VerbPhrase> for Phrase {
    fn from(value: VerbPhrase) -> Self {
        Phrase::VP(Box::new(value))
    }
}

impl From<PrepositionalPhrase> for Phrase {
    fn from(value: PrepositionalPhrase) -> Self {
        Phrase::PP(Box::new(value))
    }
}

impl From<AdjectivePhrase> for Phrase {
    fn from(value: AdjectivePhrase) -> Self {
        Phrase::AdjP(Box::new(value))
    }
}

impl From<AdverbPhrase> for Phrase {
    fn from(value: AdverbPhrase) -> Self {
        Phrase::AdvP(Box::new(value))
    }
}
