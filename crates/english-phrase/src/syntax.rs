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

pub trait ClauseForm: private::Sealed + Copy {
    fn verb_form(self) -> VerbForm;
}

pub trait NonfiniteClauseForm: ClauseForm {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finite(Tense);

impl Default for Finite {
    fn default() -> Self {
        Self(Tense::Present)
    }
}

impl Finite {
    pub fn tense(&self) -> Tense {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BareInfinitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToInfinitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gerund;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PastParticiple;

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
    modifiers: Vec<NpModifier>,
    complements: Vec<NpComplement>,
    adjuncts: Vec<NpAdjunct>,
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

    pub fn modifier<M: Into<NpModifier>>(mut self, modifier: M) -> Self {
        self.modifiers.push(modifier.into());
        self
    }

    /// ```
    /// use english_phrase::*;
    ///
    /// let phrase = np("attempt").complement(tp(vp("leave")).to_infinitive());
    /// assert_eq!(phrase.realize(), "attempt to leave");
    /// ```
    ///
    /// ```compile_fail
    /// use english_phrase::*;
    ///
    /// let _ = np("attempt").complement(tp(vp("leave")).bare_infinitive());
    /// ```
    pub fn complement<C: Into<NpComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn adjunct<A: Into<NpAdjunct>>(mut self, adjunct: A) -> Self {
        self.adjuncts.push(adjunct.into());
        self
    }

    pub fn head(&self) -> &NounEntry {
        &self.head
    }

    pub fn number(&self) -> &Number {
        &self.number
    }

    pub fn modifiers(&self) -> &[NpModifier] {
        &self.modifiers
    }

    pub fn complements(&self) -> &[NpComplement] {
        &self.complements
    }

    pub fn adjuncts(&self) -> &[NpAdjunct] {
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

    /// ```
    /// use english_phrase::*;
    ///
    /// let phrase = dp(Pronoun::She).reflexive();
    /// assert_eq!(phrase.realize(), "herself");
    /// ```
    ///
    /// ```compile_fail
    /// use english_phrase::*;
    ///
    /// let _ = dp(np("editor")).reflexive();
    /// ```
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
    modifier: Option<Box<AdverbPhrase>>,
    head: AdjectiveEntry,
    complements: Vec<ApComplement>,
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
        self.modifier = Some(Box::new(modifier));
        self
    }

    /// ```
    /// use english_phrase::*;
    ///
    /// let phrase = adjp("ready").complement(tp(vp("leave")).to_infinitive());
    /// assert_eq!(phrase.realize(), "ready to leave");
    /// ```
    ///
    /// ```compile_fail
    /// use english_phrase::*;
    ///
    /// let _ = adjp("ready").complement(tp(vp("leave")).gerund_participle());
    /// ```
    pub fn complement<C: Into<ApComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn modifier_opt(&self) -> Option<&AdverbPhrase> {
        self.modifier.as_deref()
    }

    pub fn head(&self) -> &AdjectiveEntry {
        &self.head
    }

    pub fn complements(&self) -> &[ApComplement] {
        &self.complements
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdverbPhrase {
    modifier: Option<Box<AdverbPhrase>>,
    head: AdverbEntry,
    complements: Vec<AdvpComplement>,
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
        self.modifier = Some(Box::new(modifier));
        self
    }

    pub fn complement<C: Into<AdvpComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn modifier_opt(&self) -> Option<&AdverbPhrase> {
        self.modifier.as_deref()
    }

    pub fn head(&self) -> &AdverbEntry {
        &self.head
    }

    pub fn complements(&self) -> &[AdvpComplement] {
        &self.complements
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrepositionalPhrase {
    head: PrepositionEntry,
    complement: Box<PpComplement>,
}

impl PrepositionalPhrase {
    pub fn new<C: Into<PpComplement>>(head: impl Into<PrepositionEntry>, complement: C) -> Self {
        Self {
            head: head.into(),
            complement: Box::new(complement.into()),
        }
    }

    pub fn head(&self) -> &PrepositionEntry {
        &self.head
    }

    pub fn complement(&self) -> &PpComplement {
        self.complement.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase {
    head: VerbEntry,
    complements: Vec<VpComplement>,
    adjuncts: Vec<VpAdjunct>,
}

impl VerbPhrase {
    pub fn new(head: impl Into<VerbEntry>) -> Self {
        Self {
            head: head.into(),
            complements: Vec::new(),
            adjuncts: Vec::new(),
        }
    }

    /// ```
    /// use english_phrase::*;
    ///
    /// let phrase = vp("expect").complement(tp(vp("leave")).to_infinitive());
    /// assert_eq!(phrase.realize(), "expect to leave");
    /// ```
    ///
    /// ```compile_fail
    /// use english_phrase::*;
    ///
    /// let _ = vp("say").complement(tp(vp("leave")).past());
    /// ```
    pub fn complement<C: Into<VpComplement>>(mut self, complement: C) -> Self {
        self.complements.push(complement.into());
        self
    }

    pub fn adjunct<A: Into<VpAdjunct>>(mut self, adjunct: A) -> Self {
        self.adjuncts.push(adjunct.into());
        self
    }

    pub fn head(&self) -> &VerbEntry {
        &self.head
    }

    pub fn complements(&self) -> &[VpComplement] {
        &self.complements
    }

    pub fn adjuncts(&self) -> &[VpAdjunct] {
        &self.adjuncts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensePhrase<Form: ClauseForm> {
    subject: Option<DeterminerPhrase>,
    form: Form,
    negative: bool,
    predicate: VerbPhrase,
}

impl TensePhrase<BareInfinitive> {
    pub fn new(predicate: VerbPhrase) -> Self {
        Self {
            subject: None,
            form: BareInfinitive,
            negative: false,
            predicate,
        }
    }
}

impl<Form: ClauseForm> TensePhrase<Form> {
    fn map_form<Next: ClauseForm>(self, form: Next) -> TensePhrase<Next> {
        TensePhrase {
            subject: self.subject,
            form,
            negative: self.negative,
            predicate: self.predicate,
        }
    }

    pub fn subject<S: Into<DeterminerPhrase>>(mut self, subject: S) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn present(self) -> TensePhrase<Finite> {
        self.map_form(Finite(Tense::Present))
    }

    pub fn past(self) -> TensePhrase<Finite> {
        self.map_form(Finite(Tense::Past))
    }

    pub fn bare_infinitive(self) -> TensePhrase<BareInfinitive> {
        self.map_form(BareInfinitive)
    }

    pub fn to_infinitive(self) -> TensePhrase<ToInfinitive> {
        self.map_form(ToInfinitive)
    }

    pub fn gerund_participle(self) -> TensePhrase<Gerund> {
        self.map_form(Gerund)
    }

    pub fn past_participle(self) -> TensePhrase<PastParticiple> {
        self.map_form(PastParticiple)
    }

    pub fn negative(mut self) -> Self {
        self.negative = true;
        self
    }

    pub fn subject_opt(&self) -> Option<&DeterminerPhrase> {
        self.subject.as_ref()
    }

    pub fn form(&self) -> VerbForm {
        self.form.verb_form()
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
    specifier: Option<Box<CpSpecifier>>,
    head: Complementizer,
    complement: Box<TensePhrase<Finite>>,
}

impl ComplementizerPhrase {
    pub fn new(complement: TensePhrase<Finite>) -> Self {
        Self {
            specifier: None,
            head: Complementizer::Null,
            complement: Box::new(complement),
        }
    }

    pub fn specifier<S: Into<CpSpecifier>>(mut self, specifier: S) -> Self {
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

    pub fn specifier_opt(&self) -> Option<&CpSpecifier> {
        self.specifier.as_deref()
    }

    pub fn head(&self) -> Complementizer {
        self.head
    }

    pub fn complement(&self) -> &TensePhrase<Finite> {
        &self.complement
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NpModifier {
    Adj(AdjectivePhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NpComplement {
    PP(PrepositionalPhrase),
    ToInf(TensePhrase<ToInfinitive>),
    CP(ComplementizerPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NpAdjunct {
    PP(PrepositionalPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApComplement {
    PP(PrepositionalPhrase),
    ToInf(TensePhrase<ToInfinitive>),
    CP(ComplementizerPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdvpComplement {
    PP(PrepositionalPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PpComplement {
    DP(DeterminerPhrase),
    PP(PrepositionalPhrase),
    Gerund(TensePhrase<Gerund>),
    CP(ComplementizerPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VpComplement {
    DP(DeterminerPhrase),
    PP(PrepositionalPhrase),
    AP(AdjectivePhrase),
    CP(ComplementizerPhrase),
    BareInf(TensePhrase<BareInfinitive>),
    ToInf(TensePhrase<ToInfinitive>),
    Gerund(TensePhrase<Gerund>),
    PastParticiple(TensePhrase<PastParticiple>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VpAdjunct {
    PP(PrepositionalPhrase),
    AdvP(AdverbPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CpSpecifier {
    DP(DeterminerPhrase),
    NP(NounPhrase),
    VP(VerbPhrase),
    PP(PrepositionalPhrase),
    AdjP(AdjectivePhrase),
    AdvP(AdverbPhrase),
    CP(ComplementizerPhrase),
    Finite(TensePhrase<Finite>),
    BareInf(TensePhrase<BareInfinitive>),
    ToInf(TensePhrase<ToInfinitive>),
    Gerund(TensePhrase<Gerund>),
    PastParticiple(TensePhrase<PastParticiple>),
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

/// ```
/// use english_phrase::*;
///
/// let phrase = pp("after", tp(vp("leave")).gerund_participle());
/// assert_eq!(phrase.realize(), "after leaving");
/// ```
///
/// ```compile_fail
/// use english_phrase::*;
///
/// let _ = pp("after", tp(vp("leave")).to_infinitive());
/// ```
pub fn pp<C: Into<PpComplement>>(
    head: impl Into<PrepositionEntry>,
    complement: C,
) -> PrepositionalPhrase {
    PrepositionalPhrase::new(head, complement)
}

pub fn vp(head: impl Into<VerbEntry>) -> VerbPhrase {
    VerbPhrase::new(head)
}

pub fn tp(predicate: VerbPhrase) -> TensePhrase<BareInfinitive> {
    TensePhrase::new(predicate)
}

/// ```
/// use english_phrase::*;
///
/// let phrase = cp(tp(vp("arrive")).past().subject(dp(Pronoun::She)));
/// assert_eq!(phrase.realize(), "she arrived");
/// ```
///
/// ```compile_fail
/// use english_phrase::*;
///
/// let _ = cp(tp(vp("leave")).to_infinitive());
/// ```
pub fn cp(complement: TensePhrase<Finite>) -> ComplementizerPhrase {
    ComplementizerPhrase::new(complement)
}

#[doc(hidden)]
pub trait DpHead: private::Sealed {
    type Output;

    fn into_dp(self) -> Self::Output;
}

trait DpLike: private::Sealed {
    fn into_determiner_phrase(self) -> DeterminerPhrase;
}

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

impl From<NominalDeterminerPhrase> for DeterminerPhrase {
    fn from(value: NominalDeterminerPhrase) -> Self {
        DeterminerPhrase::BareNominal(value.nominal)
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

impl From<AdjectivePhrase> for NpModifier {
    fn from(value: AdjectivePhrase) -> Self {
        Self::Adj(value)
    }
}

impl From<PrepositionalPhrase> for NpComplement {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl From<TensePhrase<ToInfinitive>> for NpComplement {
    fn from(value: TensePhrase<ToInfinitive>) -> Self {
        Self::ToInf(value)
    }
}

impl From<ComplementizerPhrase> for NpComplement {
    fn from(value: ComplementizerPhrase) -> Self {
        Self::CP(value)
    }
}

impl From<PrepositionalPhrase> for NpAdjunct {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl From<PrepositionalPhrase> for ApComplement {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl From<TensePhrase<ToInfinitive>> for ApComplement {
    fn from(value: TensePhrase<ToInfinitive>) -> Self {
        Self::ToInf(value)
    }
}

impl From<ComplementizerPhrase> for ApComplement {
    fn from(value: ComplementizerPhrase) -> Self {
        Self::CP(value)
    }
}

impl From<PrepositionalPhrase> for AdvpComplement {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl<T: DpLike> From<T> for PpComplement {
    fn from(value: T) -> Self {
        Self::DP(value.into_determiner_phrase())
    }
}

impl From<PrepositionalPhrase> for PpComplement {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl From<TensePhrase<Gerund>> for PpComplement {
    fn from(value: TensePhrase<Gerund>) -> Self {
        Self::Gerund(value)
    }
}

impl From<ComplementizerPhrase> for PpComplement {
    fn from(value: ComplementizerPhrase) -> Self {
        Self::CP(value)
    }
}

impl<T: DpLike> From<T> for VpComplement {
    fn from(value: T) -> Self {
        Self::DP(value.into_determiner_phrase())
    }
}

impl From<PrepositionalPhrase> for VpComplement {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl From<AdjectivePhrase> for VpComplement {
    fn from(value: AdjectivePhrase) -> Self {
        Self::AP(value)
    }
}

impl From<ComplementizerPhrase> for VpComplement {
    fn from(value: ComplementizerPhrase) -> Self {
        Self::CP(value)
    }
}

impl From<TensePhrase<BareInfinitive>> for VpComplement {
    fn from(value: TensePhrase<BareInfinitive>) -> Self {
        Self::BareInf(value)
    }
}

impl From<TensePhrase<ToInfinitive>> for VpComplement {
    fn from(value: TensePhrase<ToInfinitive>) -> Self {
        Self::ToInf(value)
    }
}

impl From<TensePhrase<Gerund>> for VpComplement {
    fn from(value: TensePhrase<Gerund>) -> Self {
        Self::Gerund(value)
    }
}

impl From<TensePhrase<PastParticiple>> for VpComplement {
    fn from(value: TensePhrase<PastParticiple>) -> Self {
        Self::PastParticiple(value)
    }
}

impl From<PrepositionalPhrase> for VpAdjunct {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl From<AdverbPhrase> for VpAdjunct {
    fn from(value: AdverbPhrase) -> Self {
        Self::AdvP(value)
    }
}

impl<T: DpLike> From<T> for CpSpecifier {
    fn from(value: T) -> Self {
        Self::DP(value.into_determiner_phrase())
    }
}

impl From<NounPhrase> for CpSpecifier {
    fn from(value: NounPhrase) -> Self {
        Self::NP(value)
    }
}

impl From<VerbPhrase> for CpSpecifier {
    fn from(value: VerbPhrase) -> Self {
        Self::VP(value)
    }
}

impl From<PrepositionalPhrase> for CpSpecifier {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl From<AdjectivePhrase> for CpSpecifier {
    fn from(value: AdjectivePhrase) -> Self {
        Self::AdjP(value)
    }
}

impl From<AdverbPhrase> for CpSpecifier {
    fn from(value: AdverbPhrase) -> Self {
        Self::AdvP(value)
    }
}

impl From<ComplementizerPhrase> for CpSpecifier {
    fn from(value: ComplementizerPhrase) -> Self {
        Self::CP(value)
    }
}

impl From<TensePhrase<Finite>> for CpSpecifier {
    fn from(value: TensePhrase<Finite>) -> Self {
        Self::Finite(value)
    }
}

impl From<TensePhrase<BareInfinitive>> for CpSpecifier {
    fn from(value: TensePhrase<BareInfinitive>) -> Self {
        Self::BareInf(value)
    }
}

impl From<TensePhrase<ToInfinitive>> for CpSpecifier {
    fn from(value: TensePhrase<ToInfinitive>) -> Self {
        Self::ToInf(value)
    }
}

impl From<TensePhrase<Gerund>> for CpSpecifier {
    fn from(value: TensePhrase<Gerund>) -> Self {
        Self::Gerund(value)
    }
}

impl From<TensePhrase<PastParticiple>> for CpSpecifier {
    fn from(value: TensePhrase<PastParticiple>) -> Self {
        Self::PastParticiple(value)
    }
}

impl private::Sealed for Finite {}
impl private::Sealed for BareInfinitive {}
impl private::Sealed for ToInfinitive {}
impl private::Sealed for Gerund {}
impl private::Sealed for PastParticiple {}
impl private::Sealed for DeterminerPhrase {}
impl private::Sealed for NominalDeterminerPhrase {}
impl private::Sealed for PronominalDeterminerPhrase {}
impl private::Sealed for NounPhrase {}
impl private::Sealed for VerbPhrase {}
impl<Form: ClauseForm> private::Sealed for TensePhrase<Form> {}
impl private::Sealed for ComplementizerPhrase {}
impl private::Sealed for PrepositionalPhrase {}
impl private::Sealed for AdjectivePhrase {}
impl private::Sealed for AdverbPhrase {}
impl private::Sealed for Name {}
impl private::Sealed for Pronoun {}
impl private::Sealed for NpModifier {}
impl private::Sealed for NpComplement {}
impl private::Sealed for NpAdjunct {}
impl private::Sealed for ApComplement {}
impl private::Sealed for AdvpComplement {}
impl private::Sealed for PpComplement {}
impl private::Sealed for VpComplement {}
impl private::Sealed for VpAdjunct {}
impl private::Sealed for CpSpecifier {}

impl ClauseForm for Finite {
    fn verb_form(self) -> VerbForm {
        VerbForm::Finite(self.0)
    }
}

impl ClauseForm for BareInfinitive {
    fn verb_form(self) -> VerbForm {
        VerbForm::BareInfinitive
    }
}

impl ClauseForm for ToInfinitive {
    fn verb_form(self) -> VerbForm {
        VerbForm::ToInfinitive
    }
}

impl ClauseForm for Gerund {
    fn verb_form(self) -> VerbForm {
        VerbForm::GerundParticiple
    }
}

impl ClauseForm for PastParticiple {
    fn verb_form(self) -> VerbForm {
        VerbForm::PastParticiple
    }
}

impl NonfiniteClauseForm for BareInfinitive {}
impl NonfiniteClauseForm for ToInfinitive {}
impl NonfiniteClauseForm for Gerund {}
impl NonfiniteClauseForm for PastParticiple {}
