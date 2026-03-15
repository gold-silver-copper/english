use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
use english::Number;

mod private {
    pub trait Sealed {}
}

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
    NP(Box<NounPhrase>),
    VP(Box<VerbPhrase>),
    PP(Box<PrepositionalPhrase>),
    AdjP(Box<AdjectivePhrase>),
    AdvP(Box<AdverbPhrase>),
}

#[doc(hidden)]
pub trait NpModifier: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub trait NpComplement: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub trait ApModifier: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub trait ApComplement: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub trait AdvpModifier: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub trait AdvpComplement: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub trait PpComplement: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub trait VpComplement: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub trait VpAdjunct: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeterminerHead {
    Nominal(Box<NounPhrase>),
    ProperName(String),
    Pronoun(Pronoun),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NounPhrase {
    head: NounEntry,
    number: Number,
    modifiers: Vec<Box<Phrase>>,
    complements: Vec<Box<Phrase>>,
}

impl NounPhrase {
    pub fn new(head: impl Into<NounEntry>) -> Self {
        Self {
            head: head.into(),
            number: Number::Singular,
            modifiers: Vec::new(),
            complements: Vec::new(),
        }
    }

    pub fn singular(mut self) -> Self {
        self.number = Number::Singular;
        self
    }

    pub fn plural(mut self) -> Self {
        self.number = Number::Plural;
        self
    }

    pub fn modifier<M: NpModifier>(mut self, modifier: M) -> Self {
        self.modifiers.push(Box::new(modifier.into_phrase()));
        self
    }

    pub fn complement<C: NpComplement>(mut self, complement: C) -> Self {
        self.complements.push(Box::new(complement.into_phrase()));
        self
    }

    pub fn head(&self) -> &NounEntry {
        &self.head
    }

    pub fn number(&self) -> &Number {
        &self.number
    }

    pub fn modifiers(&self) -> &[Box<Phrase>] {
        &self.modifiers
    }

    pub fn complements(&self) -> &[Box<Phrase>] {
        &self.complements
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeterminerPhrase {
    determiner: Option<Determiner>,
    head: DeterminerHead,
}

impl DeterminerPhrase {
    pub fn new(nominal: NounPhrase) -> Self {
        Self {
            determiner: None,
            head: DeterminerHead::Nominal(Box::new(nominal)),
        }
    }

    pub fn proper_name(name: impl Into<String>) -> Self {
        Self {
            determiner: None,
            head: DeterminerHead::ProperName(name.into()),
        }
    }

    pub fn pronoun(pronoun: Pronoun) -> Self {
        Self {
            determiner: None,
            head: DeterminerHead::Pronoun(pronoun),
        }
    }

    pub fn determiner(mut self, determiner: Determiner) -> Self {
        self.determiner = Some(determiner);
        self
    }

    pub fn the(self) -> Self {
        self.determiner(Determiner::The)
    }

    pub fn a(self) -> Self {
        self.determiner(Determiner::A)
    }

    pub fn an(self) -> Self {
        self.determiner(Determiner::An)
    }

    pub fn this(self) -> Self {
        self.determiner(Determiner::This)
    }

    pub fn that(self) -> Self {
        self.determiner(Determiner::That)
    }

    pub fn these(self) -> Self {
        self.determiner(Determiner::These)
    }

    pub fn those(self) -> Self {
        self.determiner(Determiner::Those)
    }

    pub fn predicate(self, predicate: VerbPhrase) -> Clause {
        Clause {
            subject: self,
            predicate,
        }
    }

    pub fn determiner_opt(&self) -> Option<Determiner> {
        self.determiner
    }

    pub fn head(&self) -> &DeterminerHead {
        &self.head
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjectivePhrase {
    modifier: Option<Box<Phrase>>,
    head: AdjectiveEntry,
    complements: Vec<Box<Phrase>>,
}

impl AdjectivePhrase {
    pub fn new(head: impl Into<AdjectiveEntry>) -> Self {
        Self {
            modifier: None,
            head: head.into(),
            complements: Vec::new(),
        }
    }

    pub fn modifier<M: ApModifier>(mut self, modifier: M) -> Self {
        self.modifier = Some(Box::new(modifier.into_phrase()));
        self
    }

    pub fn complement<C: ApComplement>(mut self, complement: C) -> Self {
        self.complements.push(Box::new(complement.into_phrase()));
        self
    }

    pub fn modifier_opt(&self) -> Option<&Phrase> {
        self.modifier.as_deref()
    }

    pub fn head(&self) -> &AdjectiveEntry {
        &self.head
    }

    pub fn complements(&self) -> &[Box<Phrase>] {
        &self.complements
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdverbPhrase {
    modifier: Option<Box<Phrase>>,
    head: AdverbEntry,
    complements: Vec<Box<Phrase>>,
}

impl AdverbPhrase {
    pub fn new(head: impl Into<AdverbEntry>) -> Self {
        Self {
            modifier: None,
            head: head.into(),
            complements: Vec::new(),
        }
    }

    pub fn modifier<M: AdvpModifier>(mut self, modifier: M) -> Self {
        self.modifier = Some(Box::new(modifier.into_phrase()));
        self
    }

    pub fn complement<C: AdvpComplement>(mut self, complement: C) -> Self {
        self.complements.push(Box::new(complement.into_phrase()));
        self
    }

    pub fn modifier_opt(&self) -> Option<&Phrase> {
        self.modifier.as_deref()
    }

    pub fn head(&self) -> &AdverbEntry {
        &self.head
    }

    pub fn complements(&self) -> &[Box<Phrase>] {
        &self.complements
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrepositionalPhrase {
    head: PrepositionEntry,
    complement: Box<Phrase>,
}

impl PrepositionalPhrase {
    pub fn new<C: PpComplement>(head: impl Into<PrepositionEntry>, complement: C) -> Self {
        Self {
            head: head.into(),
            complement: Box::new(complement.into_phrase()),
        }
    }

    pub fn head(&self) -> &PrepositionEntry {
        &self.head
    }

    pub fn complement(&self) -> &Phrase {
        self.complement.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase {
    head: VerbEntry,
    form: VerbForm,
    negative: bool,
    complements: Vec<Box<Phrase>>,
    adjuncts: Vec<Box<Phrase>>,
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

    pub fn present(mut self) -> Self {
        self.form = VerbForm::Finite(Tense::Present);
        self
    }

    pub fn past(mut self) -> Self {
        self.form = VerbForm::Finite(Tense::Past);
        self
    }

    pub fn bare_infinitive(mut self) -> Self {
        self.form = VerbForm::BareInfinitive;
        self
    }

    pub fn to_infinitive(mut self) -> Self {
        self.form = VerbForm::ToInfinitive;
        self
    }

    pub fn gerund_participle(mut self) -> Self {
        self.form = VerbForm::GerundParticiple;
        self
    }

    pub fn past_participle(mut self) -> Self {
        self.form = VerbForm::PastParticiple;
        self
    }

    pub fn negative(mut self) -> Self {
        self.negative = true;
        self
    }

    pub fn complement<C: VpComplement>(mut self, complement: C) -> Self {
        self.complements.push(Box::new(complement.into_phrase()));
        self
    }

    pub fn adjunct<A: VpAdjunct>(mut self, adjunct: A) -> Self {
        self.adjuncts.push(Box::new(adjunct.into_phrase()));
        self
    }

    pub fn head(&self) -> &VerbEntry {
        &self.head
    }

    pub fn form(&self) -> VerbForm {
        self.form
    }

    pub fn is_negative(&self) -> bool {
        self.negative
    }

    pub fn complements(&self) -> &[Box<Phrase>] {
        &self.complements
    }

    pub fn adjuncts(&self) -> &[Box<Phrase>] {
        &self.adjuncts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    subject: DeterminerPhrase,
    predicate: VerbPhrase,
}

impl Clause {
    pub fn subject(&self) -> &DeterminerPhrase {
        &self.subject
    }

    pub fn predicate(&self) -> &VerbPhrase {
        &self.predicate
    }

    pub fn sentence(self) -> Sentence {
        Sentence {
            clause: self,
            capitalize: true,
            terminal: Terminal::Period,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminal {
    Period,
    QuestionMark,
    ExclamationMark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sentence {
    clause: Clause,
    capitalize: bool,
    terminal: Terminal,
}

impl Sentence {
    pub fn clause(&self) -> &Clause {
        &self.clause
    }

    pub fn capitalize(mut self) -> Self {
        self.capitalize = true;
        self
    }

    pub fn lowercase(mut self) -> Self {
        self.capitalize = false;
        self
    }

    pub fn period(mut self) -> Self {
        self.terminal = Terminal::Period;
        self
    }

    pub fn question_mark(mut self) -> Self {
        self.terminal = Terminal::QuestionMark;
        self
    }

    pub fn exclamation_mark(mut self) -> Self {
        self.terminal = Terminal::ExclamationMark;
        self
    }

    pub fn capitalize_flag(&self) -> bool {
        self.capitalize
    }

    pub fn terminal(&self) -> Terminal {
        self.terminal
    }
}

pub fn np(head: impl Into<NounEntry>) -> NounPhrase {
    NounPhrase::new(head)
}

pub fn dp(nominal: NounPhrase) -> DeterminerPhrase {
    DeterminerPhrase::new(nominal)
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

pub fn pp<C: PpComplement>(
    head: impl Into<PrepositionEntry>,
    complement: C,
) -> PrepositionalPhrase {
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

impl From<NounPhrase> for Phrase {
    fn from(value: NounPhrase) -> Self {
        Phrase::NP(Box::new(value))
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

impl private::Sealed for DeterminerPhrase {}
impl private::Sealed for NounPhrase {}
impl private::Sealed for VerbPhrase {}
impl private::Sealed for PrepositionalPhrase {}
impl private::Sealed for AdjectivePhrase {}
impl private::Sealed for AdverbPhrase {}

impl NpModifier for AdjectivePhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl NpComplement for PrepositionalPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl NpComplement for VerbPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl ApModifier for AdverbPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl ApComplement for PrepositionalPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl ApComplement for VerbPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl AdvpModifier for AdverbPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl AdvpComplement for PrepositionalPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl PpComplement for DeterminerPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl PpComplement for PrepositionalPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl PpComplement for VerbPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpComplement for DeterminerPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpComplement for PrepositionalPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpComplement for AdjectivePhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpComplement for VerbPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpAdjunct for PrepositionalPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpAdjunct for AdverbPhrase {
    fn into_phrase(self) -> Phrase {
        self.into()
    }
}
