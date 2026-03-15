#![allow(dead_code)]

use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
use crate::syntax::Tense;
use english::Number;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum XP {
    CP(Box<CP>),
    TP(Box<TP>),
    VP(Box<VP>),
    DP(Box<DP>),
    NP(Box<NP>),
    AP(Box<AP>),
    AdvP(Box<AdvP>),
    PP(Box<PP>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CP {
    pub specifier: Option<Box<XP>>,
    pub bar: CBar,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CBar {
    pub head: CHead,
    pub complement: Box<TP>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CHead {
    Null,
    Overt(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TP {
    pub specifier: Option<Box<DP>>,
    pub bar: TBar,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TBar {
    pub head: THead,
    pub complement: Box<VP>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum THead {
    Finite(Tense),
    BareInfinitive,
    ToInfinitive,
    GerundParticiple,
    PastParticiple,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NegVBar {
    pub head: NegHead,
    pub complement: Box<VP>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NegHead {
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VP {
    pub specifier: Option<Box<DP>>,
    pub bar: VPBar,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VPBar {
    Headed(VBar),
    Negated(NegVBar),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VBar {
    pub head: VHead,
    pub complements: Vec<Box<XP>>,
    pub adjuncts: Vec<Box<XP>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VHead {
    pub entry: VerbEntry,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DP {
    pub specifier: Option<Box<DP>>,
    pub bar: DBar,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DBar {
    pub head: DHead,
    pub complement: DComplement,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DHead {
    Overt(Determiner),
    Silent(SilentDeterminer),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SilentDeterminer {
    Bare,
    ProperName,
    Pronoun,
    Trace,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DComplement {
    NP(Box<NP>),
    VP(Box<VP>),
    Trace,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NP {
    pub left_adjuncts: Vec<Box<XP>>,
    pub bar: NBar,
    pub right_adjuncts: Vec<Box<XP>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NBar {
    pub head: NHead,
    pub complements: Vec<Box<XP>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NHead {
    CommonNoun { entry: NounEntry, number: Number },
    ProperName(String),
    Pronoun(Pronoun),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AP {
    pub specifier: Option<Box<AdvP>>,
    pub bar: ABar,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ABar {
    pub head: AHead,
    pub complements: Vec<Box<XP>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AHead {
    pub entry: AdjectiveEntry,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AdvP {
    pub specifier: Option<Box<AdvP>>,
    pub bar: AdvBar,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AdvBar {
    pub head: AdvHead,
    pub complements: Vec<Box<XP>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AdvHead {
    pub entry: AdverbEntry,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PP {
    pub specifier: Option<Box<XP>>,
    pub bar: PBar,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PBar {
    pub head: PHead,
    pub complement: Option<Box<XP>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PHead {
    pub entry: PrepositionEntry,
}

impl From<CP> for XP {
    fn from(value: CP) -> Self {
        XP::CP(Box::new(value))
    }
}

impl From<TP> for XP {
    fn from(value: TP) -> Self {
        XP::TP(Box::new(value))
    }
}

impl From<VP> for XP {
    fn from(value: VP) -> Self {
        XP::VP(Box::new(value))
    }
}

impl From<DP> for XP {
    fn from(value: DP) -> Self {
        XP::DP(Box::new(value))
    }
}

impl From<NP> for XP {
    fn from(value: NP) -> Self {
        XP::NP(Box::new(value))
    }
}

impl From<AP> for XP {
    fn from(value: AP) -> Self {
        XP::AP(Box::new(value))
    }
}

impl From<AdvP> for XP {
    fn from(value: AdvP) -> Self {
        XP::AdvP(Box::new(value))
    }
}

impl From<PP> for XP {
    fn from(value: PP) -> Self {
        XP::PP(Box::new(value))
    }
}
