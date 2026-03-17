use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
use english::{Number, Person};
use std::marker::PhantomData;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Countability {
    #[default]
    Unknown,
    Count,
    Mass,
}

#[doc(hidden)]
pub trait TpForm: private::Sealed + Copy {
    fn verb_form(self) -> VerbForm;
}

#[doc(hidden)]
pub trait NonfiniteTpForm: TpForm {}

#[doc(hidden)]
pub trait NominalNumberMarker: private::Sealed + Copy {
    fn number() -> Number;
}

#[doc(hidden)]
pub trait NominalCountabilityMarker: private::Sealed + Copy {
    fn countability() -> Countability;
}

#[doc(hidden)]
pub trait PredicateGap: private::Sealed + Copy + std::fmt::Debug + PartialEq + Eq {}

#[doc(hidden)]
pub trait TpGap: private::Sealed + Copy {
    type PredicateGap: PredicateGap;

    fn subject_agreement() -> Option<(Person, Number)> {
        None
    }
}

#[doc(hidden)]
pub trait CpGap: TpGap {
    fn default_complementizer() -> Option<Complementizer> {
        None
    }

    fn default_relativizer() -> Option<Relativizer> {
        None
    }
}

#[doc(hidden)]
pub trait RelativeTpGap: CpGap {}

#[doc(hidden)]
pub trait BaseTpGap: TpGap<PredicateGap = Self> + PredicateGap {}

#[doc(hidden)]
pub trait OvertTpGap: TpGap {}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SingularNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluralNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnknownCountability;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountNoun;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MassNoun;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoGap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectGap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubjectGap<N: NominalNumberMarker>(PhantomData<N>);

impl<N: NominalNumberMarker> Default for SubjectGap<N> {
    fn default() -> Self {
        Self(PhantomData)
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Relativizer {
    #[default]
    Null,
    That,
    Who,
    Which,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CpHead {
    Content(Complementizer),
    Relative(Relativizer),
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct NounPhraseData {
    pub(crate) head: NounEntry,
    pub(crate) number: Number,
    pub(crate) countability: Countability,
    pub(crate) modifiers: Vec<NpModifier>,
    pub(crate) complements: Vec<NpComplement>,
    pub(crate) adjuncts: Vec<NpAdjunct>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NounPhrase<
    N: NominalNumberMarker = SingularNumber,
    C: NominalCountabilityMarker = UnknownCountability,
> {
    pub(crate) data: NounPhraseData,
    _marker: PhantomData<(N, C)>,
}

impl NounPhrase<SingularNumber, UnknownCountability> {
    pub fn new(head: impl Into<NounEntry>) -> Self {
        Self {
            data: NounPhraseData {
                head: head.into(),
                number: Number::Singular,
                countability: Countability::Unknown,
                modifiers: Vec::new(),
                complements: Vec::new(),
                adjuncts: Vec::new(),
            },
            _marker: PhantomData,
        }
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> NounPhrase<N, C> {
    fn map_number<Next: NominalNumberMarker>(mut self) -> NounPhrase<Next, C> {
        self.data.number = Next::number();
        NounPhrase {
            data: self.data,
            _marker: PhantomData,
        }
    }

    fn map_countability<Next: NominalCountabilityMarker>(mut self) -> NounPhrase<N, Next> {
        self.data.countability = Next::countability();
        NounPhrase {
            data: self.data,
            _marker: PhantomData,
        }
    }

    pub fn singular(self) -> NounPhrase<SingularNumber, C> {
        self.map_number()
    }

    pub fn plural(self) -> NounPhrase<PluralNumber, C> {
        self.map_number()
    }

    pub fn countable(self) -> NounPhrase<N, CountNoun> {
        self.map_countability()
    }

    pub fn mass(self) -> NounPhrase<N, MassNoun> {
        self.map_countability()
    }

    pub fn modifier<M: Into<NpModifier>>(mut self, modifier: M) -> Self {
        self.data.modifiers.push(modifier.into());
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
    pub fn complement<Cp: Into<NpComplement>>(mut self, complement: Cp) -> Self {
        self.data.complements.push(complement.into());
        self
    }

    pub fn adjunct<A: Into<NpAdjunct>>(mut self, adjunct: A) -> Self {
        self.data.adjuncts.push(adjunct.into());
        self
    }

    pub fn relative<R>(mut self, relative: R) -> Self
    where
        R: RelativeCpAttachment<N>,
    {
        self.data.adjuncts.push(relative.into_np_adjunct());
        self
    }

    pub fn head(&self) -> &NounEntry {
        &self.data.head
    }

    pub fn number(&self) -> &Number {
        &self.data.number
    }

    pub fn countability(&self) -> Countability {
        self.data.countability
    }

    pub fn modifiers(&self) -> &[NpModifier] {
        &self.data.modifiers
    }

    pub fn complements(&self) -> &[NpComplement] {
        &self.data.complements
    }

    pub fn adjuncts(&self) -> &[NpAdjunct] {
        &self.data.adjuncts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeterminerPhrase {
    BareNominal(Box<NounPhraseData>),
    DeterminedNominal {
        determiner: Determiner,
        nominal: Box<NounPhraseData>,
    },
    PossessedNominal {
        possessor: Box<DeterminerPhrase>,
        nominal: Box<NounPhraseData>,
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
pub struct NominalDeterminerPhrase<
    N: NominalNumberMarker = SingularNumber,
    C: NominalCountabilityMarker = UnknownCountability,
> {
    nominal: Box<NounPhraseData>,
    _marker: PhantomData<(N, C)>,
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> NominalDeterminerPhrase<N, C> {
    fn new(nominal: NounPhrase<N, C>) -> Self {
        Self {
            nominal: Box::new(nominal.data),
            _marker: PhantomData,
        }
    }

    fn determiner(self, determiner: Determiner) -> DeterminerPhrase {
        DeterminerPhrase::DeterminedNominal {
            determiner,
            nominal: self.nominal,
        }
    }

    pub fn the(self) -> DeterminerPhrase {
        self.determiner(Determiner::The)
    }

    pub fn possessor<P: Into<DeterminerPhrase>>(self, possessor: P) -> DeterminerPhrase {
        DeterminerPhrase::PossessedNominal {
            possessor: Box::new(possessor.into()),
            nominal: self.nominal,
        }
    }
}

impl<C: NominalCountabilityMarker> NominalDeterminerPhrase<SingularNumber, C> {
    pub fn this(self) -> DeterminerPhrase {
        self.determiner(Determiner::This)
    }

    pub fn that(self) -> DeterminerPhrase {
        self.determiner(Determiner::That)
    }
}

impl<C: NominalCountabilityMarker> NominalDeterminerPhrase<PluralNumber, C> {
    pub fn these(self) -> DeterminerPhrase {
        self.determiner(Determiner::These)
    }

    pub fn those(self) -> DeterminerPhrase {
        self.determiner(Determiner::Those)
    }
}

impl NominalDeterminerPhrase<SingularNumber, CountNoun> {
    /// ```
    /// use english_phrase::*;
    ///
    /// let phrase = dp(np("lantern").countable()).indefinite();
    /// assert_eq!(phrase.realize(), "a lantern");
    /// ```
    ///
    /// ```compile_fail
    /// use english_phrase::*;
    ///
    /// let _ = dp(np("water")).indefinite();
    /// ```
    pub fn indefinite(self) -> DeterminerPhrase {
        self.determiner(Determiner::Indefinite)
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
pub struct VerbPhrase<G: PredicateGap = NoGap> {
    head: VerbEntry,
    complements: Vec<VpComplement>,
    adjuncts: Vec<VpAdjunct>,
    _gap: PhantomData<G>,
}

impl VerbPhrase<NoGap> {
    pub fn new(head: impl Into<VerbEntry>) -> Self {
        Self {
            head: head.into(),
            complements: Vec::new(),
            adjuncts: Vec::new(),
            _gap: PhantomData,
        }
    }

    pub fn object_gap(mut self) -> VerbPhrase<ObjectGap> {
        self.complements.push(VpComplement::GapObject);
        VerbPhrase {
            head: self.head,
            complements: self.complements,
            adjuncts: self.adjuncts,
            _gap: PhantomData,
        }
    }
}

impl<G: PredicateGap> VerbPhrase<G> {
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

#[derive(Clone)]
struct VerbProjection<G: TpGap = NoGap> {
    subject: Option<DeterminerPhrase>,
    predicate: VerbPhrase<G::PredicateGap>,
}

impl<G: BaseTpGap> VerbProjection<G> {
    fn new(predicate: VerbPhrase<G>) -> Self {
        Self {
            subject: None,
            predicate,
        }
    }
}

impl<G: TpGap> VerbProjection<G> {
    fn subject_opt(&self) -> Option<&DeterminerPhrase> {
        self.subject.as_ref()
    }

    fn predicate(&self) -> &VerbPhrase<G::PredicateGap> {
        &self.predicate
    }
}

impl<G: TpGap> std::fmt::Debug for VerbProjection<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerbProjection")
            .field("subject", &self.subject)
            .field("predicate", &self.predicate)
            .finish()
    }
}

impl<G: TpGap> PartialEq for VerbProjection<G> {
    fn eq(&self, other: &Self) -> bool {
        self.subject == other.subject && self.predicate == other.predicate
    }
}

impl<G: OvertTpGap> VerbProjection<G> {
    fn subject<S: Into<DeterminerPhrase>>(mut self, subject: S) -> Self {
        self.subject = Some(subject.into());
        self
    }
}

impl VerbProjection<NoGap> {
    fn subject_gap<N: NominalNumberMarker>(self) -> VerbProjection<SubjectGap<N>> {
        VerbProjection {
            subject: None,
            predicate: self.predicate,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensePhrase<Form: TpForm, G: TpGap = NoGap> {
    projection: VerbProjection<G>,
    form: Form,
    negative: bool,
}

impl<G: BaseTpGap> TensePhrase<BareInfinitive, G> {
    pub fn new(predicate: VerbPhrase<G>) -> Self {
        Self {
            projection: VerbProjection::new(predicate),
            form: BareInfinitive,
            negative: false,
        }
    }
}

impl<Form: TpForm, G: TpGap> TensePhrase<Form, G> {
    fn map_form<Next: TpForm>(self, form: Next) -> TensePhrase<Next, G> {
        TensePhrase {
            projection: self.projection,
            form,
            negative: self.negative,
        }
    }

    pub fn present(self) -> TensePhrase<Finite, G> {
        self.map_form(Finite(Tense::Present))
    }

    pub fn past(self) -> TensePhrase<Finite, G> {
        self.map_form(Finite(Tense::Past))
    }

    pub fn bare_infinitive(self) -> TensePhrase<BareInfinitive, G> {
        self.map_form(BareInfinitive)
    }

    pub fn to_infinitive(self) -> TensePhrase<ToInfinitive, G> {
        self.map_form(ToInfinitive)
    }

    pub fn gerund_participle(self) -> TensePhrase<Gerund, G> {
        self.map_form(Gerund)
    }

    pub fn past_participle(self) -> TensePhrase<PastParticiple, G> {
        self.map_form(PastParticiple)
    }

    pub fn negative(mut self) -> Self {
        self.negative = true;
        self
    }

    pub fn subject_opt(&self) -> Option<&DeterminerPhrase> {
        self.projection.subject_opt()
    }

    pub fn form(&self) -> VerbForm {
        self.form.verb_form()
    }

    pub fn is_negative(&self) -> bool {
        self.negative
    }

    pub fn predicate(&self) -> &VerbPhrase<G::PredicateGap> {
        self.projection.predicate()
    }
}

impl<Form: TpForm, G: OvertTpGap> TensePhrase<Form, G> {
    pub fn subject<S: Into<DeterminerPhrase>>(mut self, subject: S) -> Self {
        self.projection = self.projection.subject(subject);
        self
    }
}

impl<Form: TpForm> TensePhrase<Form, NoGap> {
    pub fn subject_gap<N: NominalNumberMarker>(self) -> TensePhrase<Form, SubjectGap<N>> {
        TensePhrase {
            projection: self.projection.subject_gap(),
            form: self.form,
            negative: self.negative,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplementizerPhrase<G: CpGap = NoGap> {
    head: CpHead,
    complement: Box<TensePhrase<Finite, G>>,
}

impl<G: CpGap> ComplementizerPhrase<G> {
    pub fn new(complement: TensePhrase<Finite, G>) -> Self {
        let head = if let Some(head) = G::default_complementizer() {
            CpHead::Content(head)
        } else {
            CpHead::Relative(
                G::default_relativizer()
                    .expect("CP gaps must define either a complementizer or a relativizer"),
            )
        };

        Self {
            head,
            complement: Box::new(complement),
        }
    }

    pub fn complement(&self) -> &TensePhrase<Finite, G> {
        &self.complement
    }
}

impl ComplementizerPhrase<NoGap> {
    pub fn complementizer(mut self, head: Complementizer) -> Self {
        self.head = CpHead::Content(head);
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

    pub fn head(&self) -> Complementizer {
        match self.head {
            CpHead::Content(head) => head,
            CpHead::Relative(_) => unreachable!("content CPs only carry complementizers"),
        }
    }
}

#[doc(hidden)]
pub trait RelativeCpAttachment<N: NominalNumberMarker>: private::Sealed {
    fn into_np_adjunct(self) -> NpAdjunct;
}

impl<G: RelativeTpGap> ComplementizerPhrase<G> {
    pub fn relativizer(mut self, head: Relativizer) -> Self {
        self.head = CpHead::Relative(head);
        self
    }

    pub fn null_rel(self) -> Self {
        self.relativizer(Relativizer::Null)
    }

    pub fn that(self) -> Self {
        self.relativizer(Relativizer::That)
    }

    pub fn who(self) -> Self {
        self.relativizer(Relativizer::Who)
    }

    pub fn which(self) -> Self {
        self.relativizer(Relativizer::Which)
    }

    pub fn head(&self) -> Relativizer {
        match self.head {
            CpHead::Relative(head) => head,
            CpHead::Content(_) => unreachable!("relative CPs only carry relativizers"),
        }
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
    CP(ComplementizerPhrase<NoGap>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NpAdjunct {
    PP(PrepositionalPhrase),
    RelativeObject(ComplementizerPhrase<ObjectGap>),
    RelativeSubjectSingular(ComplementizerPhrase<SubjectGap<SingularNumber>>),
    RelativeSubjectPlural(ComplementizerPhrase<SubjectGap<PluralNumber>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApComplement {
    PP(PrepositionalPhrase),
    ToInf(TensePhrase<ToInfinitive>),
    CP(ComplementizerPhrase<NoGap>),
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
    CP(ComplementizerPhrase<NoGap>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VpComplement {
    DP(DeterminerPhrase),
    PP(PrepositionalPhrase),
    AP(AdjectivePhrase),
    CP(ComplementizerPhrase<NoGap>),
    BareInf(TensePhrase<BareInfinitive>),
    ToInf(TensePhrase<ToInfinitive>),
    Gerund(TensePhrase<Gerund>),
    PastParticiple(TensePhrase<PastParticiple>),
    GapObject,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VpAdjunct {
    PP(PrepositionalPhrase),
    AdvP(AdverbPhrase),
}

pub fn np(head: impl Into<NounEntry>) -> NounPhrase<SingularNumber, UnknownCountability> {
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

pub fn vp(head: impl Into<VerbEntry>) -> VerbPhrase<NoGap> {
    VerbPhrase::new(head)
}

pub fn tp<G>(predicate: VerbPhrase<G>) -> TensePhrase<BareInfinitive, G>
where
    G: BaseTpGap,
{
    TensePhrase::new(predicate)
}

/// ```
/// use english_phrase::*;
///
/// let phrase = cp(tp(vp("arrive")).past().subject(dp(Pronoun::She))).that();
/// assert_eq!(phrase.realize(), "that she arrived");
/// ```
///
/// ```compile_fail
/// use english_phrase::*;
///
/// let _ = cp(tp(vp("leave")).to_infinitive());
/// ```
pub fn cp<G: CpGap>(complement: TensePhrase<Finite, G>) -> ComplementizerPhrase<G> {
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

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> DpHead
    for NominalDeterminerPhrase<N, C>
{
    type Output = Self;

    fn into_dp(self) -> Self::Output {
        self
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> DpHead for NounPhrase<N, C> {
    type Output = NominalDeterminerPhrase<N, C>;

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

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> DpLike
    for NominalDeterminerPhrase<N, C>
{
    fn into_determiner_phrase(self) -> DeterminerPhrase {
        self.into()
    }
}

impl DpLike for PronominalDeterminerPhrase {
    fn into_determiner_phrase(self) -> DeterminerPhrase {
        self.into()
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> From<NominalDeterminerPhrase<N, C>>
    for DeterminerPhrase
{
    fn from(value: NominalDeterminerPhrase<N, C>) -> Self {
        DeterminerPhrase::BareNominal(value.nominal)
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> From<NounPhrase<N, C>>
    for DeterminerPhrase
{
    fn from(value: NounPhrase<N, C>) -> Self {
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

impl From<ComplementizerPhrase<NoGap>> for NpComplement {
    fn from(value: ComplementizerPhrase<NoGap>) -> Self {
        Self::CP(value)
    }
}

impl From<PrepositionalPhrase> for NpAdjunct {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl<N: NominalNumberMarker> RelativeCpAttachment<N> for ComplementizerPhrase<ObjectGap> {
    fn into_np_adjunct(self) -> NpAdjunct {
        NpAdjunct::RelativeObject(self)
    }
}

impl RelativeCpAttachment<SingularNumber> for ComplementizerPhrase<SubjectGap<SingularNumber>> {
    fn into_np_adjunct(self) -> NpAdjunct {
        NpAdjunct::RelativeSubjectSingular(self)
    }
}

impl RelativeCpAttachment<PluralNumber> for ComplementizerPhrase<SubjectGap<PluralNumber>> {
    fn into_np_adjunct(self) -> NpAdjunct {
        NpAdjunct::RelativeSubjectPlural(self)
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

impl From<ComplementizerPhrase<NoGap>> for ApComplement {
    fn from(value: ComplementizerPhrase<NoGap>) -> Self {
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

impl From<ComplementizerPhrase<NoGap>> for PpComplement {
    fn from(value: ComplementizerPhrase<NoGap>) -> Self {
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

impl From<ComplementizerPhrase<NoGap>> for VpComplement {
    fn from(value: ComplementizerPhrase<NoGap>) -> Self {
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

impl private::Sealed for Finite {}
impl private::Sealed for BareInfinitive {}
impl private::Sealed for ToInfinitive {}
impl private::Sealed for Gerund {}
impl private::Sealed for PastParticiple {}
impl private::Sealed for SingularNumber {}
impl private::Sealed for PluralNumber {}
impl private::Sealed for UnknownCountability {}
impl private::Sealed for CountNoun {}
impl private::Sealed for MassNoun {}
impl private::Sealed for NoGap {}
impl private::Sealed for ObjectGap {}
impl<N: NominalNumberMarker> private::Sealed for SubjectGap<N> {}
impl private::Sealed for DeterminerPhrase {}
impl<N: NominalNumberMarker, C: NominalCountabilityMarker> private::Sealed
    for NominalDeterminerPhrase<N, C>
{
}
impl private::Sealed for PronominalDeterminerPhrase {}
impl<N: NominalNumberMarker, C: NominalCountabilityMarker> private::Sealed for NounPhrase<N, C> {}
impl<G: PredicateGap> private::Sealed for VerbPhrase<G> {}
impl<Form: TpForm, G: TpGap> private::Sealed for TensePhrase<Form, G> {}
impl<G: CpGap> private::Sealed for ComplementizerPhrase<G> {}
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
impl TpForm for Finite {
    fn verb_form(self) -> VerbForm {
        VerbForm::Finite(self.0)
    }
}

impl TpForm for BareInfinitive {
    fn verb_form(self) -> VerbForm {
        VerbForm::BareInfinitive
    }
}

impl TpForm for ToInfinitive {
    fn verb_form(self) -> VerbForm {
        VerbForm::ToInfinitive
    }
}

impl TpForm for Gerund {
    fn verb_form(self) -> VerbForm {
        VerbForm::GerundParticiple
    }
}

impl TpForm for PastParticiple {
    fn verb_form(self) -> VerbForm {
        VerbForm::PastParticiple
    }
}

impl NonfiniteTpForm for BareInfinitive {}
impl NonfiniteTpForm for ToInfinitive {}
impl NonfiniteTpForm for Gerund {}
impl NonfiniteTpForm for PastParticiple {}

impl NominalNumberMarker for SingularNumber {
    fn number() -> Number {
        Number::Singular
    }
}

impl NominalNumberMarker for PluralNumber {
    fn number() -> Number {
        Number::Plural
    }
}

impl NominalCountabilityMarker for UnknownCountability {
    fn countability() -> Countability {
        Countability::Unknown
    }
}

impl NominalCountabilityMarker for CountNoun {
    fn countability() -> Countability {
        Countability::Count
    }
}

impl NominalCountabilityMarker for MassNoun {
    fn countability() -> Countability {
        Countability::Mass
    }
}

impl PredicateGap for NoGap {}
impl PredicateGap for ObjectGap {}

impl BaseTpGap for NoGap {}
impl BaseTpGap for ObjectGap {}

impl OvertTpGap for NoGap {}
impl OvertTpGap for ObjectGap {}

impl TpGap for NoGap {
    type PredicateGap = NoGap;
}

impl TpGap for ObjectGap {
    type PredicateGap = ObjectGap;
}

impl<N: NominalNumberMarker> TpGap for SubjectGap<N> {
    type PredicateGap = NoGap;

    fn subject_agreement() -> Option<(Person, Number)> {
        Some((Person::Third, N::number()))
    }
}

impl CpGap for NoGap {
    fn default_complementizer() -> Option<Complementizer> {
        Some(Complementizer::Null)
    }
}

impl CpGap for ObjectGap {
    fn default_relativizer() -> Option<Relativizer> {
        Some(Relativizer::Null)
    }
}

impl<N: NominalNumberMarker> CpGap for SubjectGap<N> {
    fn default_relativizer() -> Option<Relativizer> {
        Some(Relativizer::Null)
    }
}

impl RelativeTpGap for ObjectGap {}
impl<N: NominalNumberMarker> RelativeTpGap for SubjectGap<N> {}
