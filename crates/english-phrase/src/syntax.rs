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
    CP(Box<ComplementizerPhrase>),
    TP(Box<TensePhrase>),
    DP(Box<DeterminerPhrase>),
    NP(Box<NounPhrase>),
    VP(Box<VerbPhrase>),
    PP(Box<PrepositionalPhrase>),
    AdjP(Box<AdjectivePhrase>),
    AdvP(Box<AdverbPhrase>),
}

#[doc(hidden)]
pub trait DpHead: private::Sealed {
    type Output;

    fn into_dp(self) -> Self::Output;
}

#[doc(hidden)]
pub trait IntoSlot<S>: private::Sealed {
    fn into_phrase(self) -> Phrase;
}

#[doc(hidden)]
pub enum NpModifierSlot {}

#[doc(hidden)]
pub enum NpComplementSlot {}

#[doc(hidden)]
pub enum NpAdjunctSlot {}

#[doc(hidden)]
pub enum ApModifierSlot {}

#[doc(hidden)]
pub enum ApComplementSlot {}

#[doc(hidden)]
pub enum AdvpModifierSlot {}

#[doc(hidden)]
pub enum AdvpComplementSlot {}

#[doc(hidden)]
pub enum PpComplementSlot {}

#[doc(hidden)]
pub enum VpComplementSlot {}

#[doc(hidden)]
pub enum VpAdjunctSlot {}

trait DpLike: private::Sealed {
    fn into_determiner_phrase(self) -> DeterminerPhrase;
}

trait NpModifierLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait NpComplementLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait NpAdjunctLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait ApModifierLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait ApComplementLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait AdvpModifierLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait AdvpComplementLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait PpComplementLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait VpComplementLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

trait VpAdjunctLike: private::Sealed {
    fn into_slot_phrase(self) -> Phrase;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

impl Name {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Complementizer {
    #[default]
    Null,
    That,
    Whether,
    If,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NounPhrase {
    head: NounEntry,
    number: Number,
    modifiers: Vec<Box<Phrase>>,
    complements: Vec<Box<Phrase>>,
    adjuncts: Vec<Box<Phrase>>,
}

impl NounPhrase {
    pub fn new(head: impl Into<NounEntry>) -> Self {
        Self {
            head: head.into(),
            number: Number::Singular,
            modifiers: Vec::new(),
            complements: Vec::new(),
            adjuncts: Vec::new(),
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

    pub fn modifier<M: IntoSlot<NpModifierSlot>>(mut self, modifier: M) -> Self {
        self.modifiers.push(Box::new(modifier.into_phrase()));
        self
    }

    pub fn complement<C: IntoSlot<NpComplementSlot>>(mut self, complement: C) -> Self {
        self.complements.push(Box::new(complement.into_phrase()));
        self
    }

    pub fn adjunct<A: IntoSlot<NpAdjunctSlot>>(mut self, adjunct: A) -> Self {
        self.adjuncts.push(Box::new(adjunct.into_phrase()));
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

    pub fn adjuncts(&self) -> &[Box<Phrase>] {
        &self.adjuncts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeterminerPhrase {
    BareNominal(Box<NounPhrase>),
    DeterminedNominal {
        determiner: Determiner,
        nominal: Box<NounPhrase>,
    },
    PossessedNominal {
        possessor: Box<DeterminerPhrase>,
        nominal: Box<NounPhrase>,
    },
    ProperName(String),
    Pronoun {
        pronoun: Pronoun,
        reflexive: bool,
    },
}

impl DeterminerPhrase {
    pub fn proper_name(name: impl Into<String>) -> Self {
        Self::ProperName(name.into())
    }

    pub fn nominal_opt(&self) -> Option<&NounPhrase> {
        match self {
            Self::BareNominal(nominal)
            | Self::DeterminedNominal { nominal, .. }
            | Self::PossessedNominal { nominal, .. } => Some(nominal),
            Self::ProperName(_) | Self::Pronoun { .. } => None,
        }
    }

    pub fn determiner_opt(&self) -> Option<Determiner> {
        match self {
            Self::DeterminedNominal { determiner, .. } => Some(*determiner),
            Self::BareNominal(_)
            | Self::PossessedNominal { .. }
            | Self::ProperName(_)
            | Self::Pronoun { .. } => None,
        }
    }

    pub fn possessor_opt(&self) -> Option<&DeterminerPhrase> {
        match self {
            Self::PossessedNominal { possessor, .. } => Some(possessor),
            Self::BareNominal(_)
            | Self::DeterminedNominal { .. }
            | Self::ProperName(_)
            | Self::Pronoun { .. } => None,
        }
    }

    pub fn proper_name_opt(&self) -> Option<&str> {
        match self {
            Self::ProperName(name) => Some(name),
            Self::BareNominal(_)
            | Self::DeterminedNominal { .. }
            | Self::PossessedNominal { .. }
            | Self::Pronoun { .. } => None,
        }
    }

    pub fn pronoun_opt(&self) -> Option<Pronoun> {
        match self {
            Self::Pronoun { pronoun, .. } => Some(*pronoun),
            Self::BareNominal(_)
            | Self::DeterminedNominal { .. }
            | Self::PossessedNominal { .. }
            | Self::ProperName(_) => None,
        }
    }

    pub fn is_reflexive(&self) -> bool {
        matches!(
            self,
            Self::Pronoun {
                reflexive: true,
                ..
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NominalDeterminerPhrase {
    nominal: Box<NounPhrase>,
}

impl NominalDeterminerPhrase {
    pub fn new(nominal: NounPhrase) -> Self {
        Self {
            nominal: Box::new(nominal),
        }
    }

    pub fn determiner(self, determiner: Determiner) -> DeterminerPhrase {
        DeterminerPhrase::DeterminedNominal {
            determiner,
            nominal: self.nominal,
        }
    }

    pub fn the(self) -> DeterminerPhrase {
        self.determiner(Determiner::The)
    }

    pub fn indefinite(self) -> DeterminerPhrase {
        self.determiner(Determiner::Indefinite)
    }

    pub fn this(self) -> DeterminerPhrase {
        self.determiner(Determiner::This)
    }

    pub fn that(self) -> DeterminerPhrase {
        self.determiner(Determiner::That)
    }

    pub fn these(self) -> DeterminerPhrase {
        self.determiner(Determiner::These)
    }

    pub fn those(self) -> DeterminerPhrase {
        self.determiner(Determiner::Those)
    }

    pub fn possessor<P: Into<DeterminerPhrase>>(self, possessor: P) -> DeterminerPhrase {
        DeterminerPhrase::PossessedNominal {
            possessor: Box::new(possessor.into()),
            nominal: self.nominal,
        }
    }

    pub fn nominal(&self) -> &NounPhrase {
        &self.nominal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PronominalDeterminerPhrase {
    pronoun: Pronoun,
    reflexive: bool,
}

impl PronominalDeterminerPhrase {
    pub fn new(pronoun: Pronoun) -> Self {
        Self {
            pronoun,
            reflexive: false,
        }
    }

    pub fn reflexive(mut self) -> Self {
        self.reflexive = true;
        self
    }

    pub fn pronoun(&self) -> Pronoun {
        self.pronoun
    }

    pub fn is_reflexive(&self) -> bool {
        self.reflexive
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

    pub fn modifier<M: IntoSlot<ApModifierSlot>>(mut self, modifier: M) -> Self {
        self.modifier = Some(Box::new(modifier.into_phrase()));
        self
    }

    pub fn complement<C: IntoSlot<ApComplementSlot>>(mut self, complement: C) -> Self {
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

    pub fn modifier<M: IntoSlot<AdvpModifierSlot>>(mut self, modifier: M) -> Self {
        self.modifier = Some(Box::new(modifier.into_phrase()));
        self
    }

    pub fn complement<C: IntoSlot<AdvpComplementSlot>>(mut self, complement: C) -> Self {
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
    pub fn new<C: IntoSlot<PpComplementSlot>>(
        head: impl Into<PrepositionEntry>,
        complement: C,
    ) -> Self {
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
    complements: Vec<Box<Phrase>>,
    adjuncts: Vec<Box<Phrase>>,
}

impl VerbPhrase {
    pub fn new(head: impl Into<VerbEntry>) -> Self {
        Self {
            head: head.into(),
            complements: Vec::new(),
            adjuncts: Vec::new(),
        }
    }

    pub fn complement<C: IntoSlot<VpComplementSlot>>(mut self, complement: C) -> Self {
        self.complements.push(Box::new(complement.into_phrase()));
        self
    }

    pub fn adjunct<A: IntoSlot<VpAdjunctSlot>>(mut self, adjunct: A) -> Self {
        self.adjuncts.push(Box::new(adjunct.into_phrase()));
        self
    }

    pub fn head(&self) -> &VerbEntry {
        &self.head
    }

    pub fn complements(&self) -> &[Box<Phrase>] {
        &self.complements
    }

    pub fn adjuncts(&self) -> &[Box<Phrase>] {
        &self.adjuncts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensePhrase {
    subject: Option<DeterminerPhrase>,
    form: VerbForm,
    negative: bool,
    predicate: VerbPhrase,
}

impl TensePhrase {
    pub fn new(predicate: VerbPhrase) -> Self {
        Self {
            subject: None,
            form: VerbForm::BareInfinitive,
            negative: false,
            predicate,
        }
    }

    pub fn subject<S: Into<DeterminerPhrase>>(mut self, subject: S) -> Self {
        self.subject = Some(subject.into());
        self
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

    pub fn subject_opt(&self) -> Option<&DeterminerPhrase> {
        self.subject.as_ref()
    }

    pub fn form(&self) -> VerbForm {
        self.form
    }

    pub fn is_negative(&self) -> bool {
        self.negative
    }

    pub fn predicate(&self) -> &VerbPhrase {
        &self.predicate
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplementizerPhrase {
    specifier: Option<Box<Phrase>>,
    head: Complementizer,
    complement: Box<TensePhrase>,
}

impl ComplementizerPhrase {
    pub fn new(complement: TensePhrase) -> Self {
        Self {
            specifier: None,
            head: Complementizer::Null,
            complement: Box::new(complement),
        }
    }

    pub fn specifier<S: Into<Phrase>>(mut self, specifier: S) -> Self {
        self.specifier = Some(Box::new(specifier.into()));
        self
    }

    pub fn complementizer(mut self, head: Complementizer) -> Self {
        self.head = head;
        self
    }

    pub fn null_c(self) -> Self {
        self.complementizer(Complementizer::Null)
    }

    pub fn that(self) -> Self {
        self.complementizer(Complementizer::That)
    }

    pub fn whether(self) -> Self {
        self.complementizer(Complementizer::Whether)
    }

    pub fn if_(self) -> Self {
        self.complementizer(Complementizer::If)
    }

    pub fn specifier_opt(&self) -> Option<&Phrase> {
        self.specifier.as_deref()
    }

    pub fn head(&self) -> Complementizer {
        self.head
    }

    pub fn complement(&self) -> &TensePhrase {
        &self.complement
    }
}

pub fn np(head: impl Into<NounEntry>) -> NounPhrase {
    NounPhrase::new(head)
}

pub fn dp<H: DpHead>(head: H) -> H::Output {
    head.into_dp()
}

pub fn name(text: impl Into<String>) -> Name {
    Name::new(text)
}

pub fn adjp(head: impl Into<AdjectiveEntry>) -> AdjectivePhrase {
    AdjectivePhrase::new(head)
}

pub fn advp(head: impl Into<AdverbEntry>) -> AdverbPhrase {
    AdverbPhrase::new(head)
}

pub fn pp<C: IntoSlot<PpComplementSlot>>(
    head: impl Into<PrepositionEntry>,
    complement: C,
) -> PrepositionalPhrase {
    PrepositionalPhrase::new(head, complement)
}

pub fn vp(head: impl Into<VerbEntry>) -> VerbPhrase {
    VerbPhrase::new(head)
}

pub fn tp(predicate: VerbPhrase) -> TensePhrase {
    TensePhrase::new(predicate)
}

pub fn cp(complement: TensePhrase) -> ComplementizerPhrase {
    ComplementizerPhrase::new(complement)
}

impl From<ComplementizerPhrase> for Phrase {
    fn from(value: ComplementizerPhrase) -> Self {
        Phrase::CP(Box::new(value))
    }
}

impl From<TensePhrase> for Phrase {
    fn from(value: TensePhrase) -> Self {
        Phrase::TP(Box::new(value))
    }
}

impl From<DeterminerPhrase> for Phrase {
    fn from(value: DeterminerPhrase) -> Self {
        Phrase::DP(Box::new(value))
    }
}

impl From<NominalDeterminerPhrase> for DeterminerPhrase {
    fn from(value: NominalDeterminerPhrase) -> Self {
        DeterminerPhrase::BareNominal(value.nominal)
    }
}

impl From<NominalDeterminerPhrase> for Phrase {
    fn from(value: NominalDeterminerPhrase) -> Self {
        Phrase::from(DeterminerPhrase::from(value))
    }
}

impl From<NounPhrase> for DeterminerPhrase {
    fn from(value: NounPhrase) -> Self {
        NominalDeterminerPhrase::new(value).into()
    }
}

impl From<PronominalDeterminerPhrase> for DeterminerPhrase {
    fn from(value: PronominalDeterminerPhrase) -> Self {
        DeterminerPhrase::Pronoun {
            pronoun: value.pronoun,
            reflexive: value.reflexive,
        }
    }
}

impl From<PronominalDeterminerPhrase> for Phrase {
    fn from(value: PronominalDeterminerPhrase) -> Self {
        Phrase::from(DeterminerPhrase::from(value))
    }
}

impl From<Pronoun> for DeterminerPhrase {
    fn from(value: Pronoun) -> Self {
        PronominalDeterminerPhrase::new(value).into()
    }
}

impl From<Name> for DeterminerPhrase {
    fn from(value: Name) -> Self {
        DeterminerPhrase::proper_name(value.0)
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
impl private::Sealed for NominalDeterminerPhrase {}
impl private::Sealed for PronominalDeterminerPhrase {}
impl private::Sealed for NounPhrase {}
impl private::Sealed for VerbPhrase {}
impl private::Sealed for TensePhrase {}
impl private::Sealed for ComplementizerPhrase {}
impl private::Sealed for PrepositionalPhrase {}
impl private::Sealed for AdjectivePhrase {}
impl private::Sealed for AdverbPhrase {}
impl private::Sealed for Name {}
impl private::Sealed for Pronoun {}

impl DpHead for DeterminerPhrase {
    type Output = Self;

    fn into_dp(self) -> Self::Output {
        self
    }
}

impl DpHead for NominalDeterminerPhrase {
    type Output = Self;

    fn into_dp(self) -> Self::Output {
        self
    }
}

impl DpHead for NounPhrase {
    type Output = NominalDeterminerPhrase;

    fn into_dp(self) -> Self::Output {
        NominalDeterminerPhrase::new(self)
    }
}

impl DpHead for PronominalDeterminerPhrase {
    type Output = Self;

    fn into_dp(self) -> Self::Output {
        self
    }
}

impl DpHead for Pronoun {
    type Output = PronominalDeterminerPhrase;

    fn into_dp(self) -> Self::Output {
        PronominalDeterminerPhrase::new(self)
    }
}

impl DpHead for Name {
    type Output = DeterminerPhrase;

    fn into_dp(self) -> Self::Output {
        self.into()
    }
}

impl DpLike for DeterminerPhrase {
    fn into_determiner_phrase(self) -> DeterminerPhrase {
        self
    }
}

impl DpLike for NominalDeterminerPhrase {
    fn into_determiner_phrase(self) -> DeterminerPhrase {
        self.into()
    }
}

impl DpLike for PronominalDeterminerPhrase {
    fn into_determiner_phrase(self) -> DeterminerPhrase {
        self.into()
    }
}

impl NpModifierLike for AdjectivePhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl NpComplementLike for PrepositionalPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl NpComplementLike for TensePhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl NpComplementLike for ComplementizerPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl NpAdjunctLike for PrepositionalPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl ApModifierLike for AdverbPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl ApComplementLike for PrepositionalPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl ApComplementLike for TensePhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl ApComplementLike for ComplementizerPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl AdvpModifierLike for AdverbPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl AdvpComplementLike for PrepositionalPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl<T: DpLike> PpComplementLike for T {
    fn into_slot_phrase(self) -> Phrase {
        self.into_determiner_phrase().into()
    }
}

impl PpComplementLike for PrepositionalPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl PpComplementLike for TensePhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl PpComplementLike for ComplementizerPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl<T: DpLike> VpComplementLike for T {
    fn into_slot_phrase(self) -> Phrase {
        self.into_determiner_phrase().into()
    }
}

impl VpComplementLike for PrepositionalPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpComplementLike for AdjectivePhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpComplementLike for TensePhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpComplementLike for ComplementizerPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpAdjunctLike for PrepositionalPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl VpAdjunctLike for AdverbPhrase {
    fn into_slot_phrase(self) -> Phrase {
        self.into()
    }
}

impl<T: NpModifierLike> IntoSlot<NpModifierSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: NpComplementLike> IntoSlot<NpComplementSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: NpAdjunctLike> IntoSlot<NpAdjunctSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: ApModifierLike> IntoSlot<ApModifierSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: ApComplementLike> IntoSlot<ApComplementSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: AdvpModifierLike> IntoSlot<AdvpModifierSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: AdvpComplementLike> IntoSlot<AdvpComplementSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: PpComplementLike> IntoSlot<PpComplementSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: VpComplementLike> IntoSlot<VpComplementSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}

impl<T: VpAdjunctLike> IntoSlot<VpAdjunctSlot> for T {
    fn into_phrase(self) -> Phrase {
        self.into_slot_phrase()
    }
}
