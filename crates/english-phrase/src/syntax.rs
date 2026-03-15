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
    determiner: Option<Determiner>,
    head: DeterminerHead,
    number: Number,
    modifiers: Vec<Box<Phrase>>,
    complements: Vec<Box<Phrase>>,
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

    pub fn determiner(mut self, determiner: Determiner) -> Self {
        self.determiner = Some(determiner);
        self
    }

    pub fn singular(mut self) -> Self {
        self.number = Number::Singular;
        self
    }

    pub fn plural(mut self) -> Self {
        self.number = Number::Plural;
        self
    }

    pub fn modifier(mut self, modifier: impl Into<Phrase>) -> Self {
        self.modifiers.push(Box::new(modifier.into()));
        self
    }

    pub fn complement(mut self, complement: impl Into<Phrase>) -> Self {
        self.complements.push(Box::new(complement.into()));
        self
    }

    pub fn determiner_opt(&self) -> Option<Determiner> {
        self.determiner
    }

    pub fn head(&self) -> &DeterminerHead {
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

    pub fn modifier(mut self, modifier: AdverbPhrase) -> Self {
        self.modifier = Some(Box::new(modifier.into()));
        self
    }

    pub fn complement(mut self, complement: impl Into<Phrase>) -> Self {
        self.complements.push(Box::new(complement.into()));
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

    pub fn modifier(mut self, modifier: AdverbPhrase) -> Self {
        self.modifier = Some(Box::new(modifier.into()));
        self
    }

    pub fn complement(mut self, complement: impl Into<Phrase>) -> Self {
        self.complements.push(Box::new(complement.into()));
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
    pub fn new(head: impl Into<PrepositionEntry>, complement: impl Into<Phrase>) -> Self {
        Self {
            head: head.into(),
            complement: Box::new(complement.into()),
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

    pub fn finite(head: impl Into<VerbEntry>, tense: Tense) -> Self {
        Self {
            form: VerbForm::Finite(tense),
            ..Self::new(head)
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

    pub fn complement(mut self, complement: impl Into<Phrase>) -> Self {
        self.complements.push(Box::new(complement.into()));
        self
    }

    pub fn adjunct(mut self, adjunct: impl Into<Phrase>) -> Self {
        self.adjuncts.push(Box::new(adjunct.into()));
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
