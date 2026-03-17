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
pub trait AgreementMarker: private::Sealed + Copy {
    fn agreement() -> Option<(Person, Number)> {
        None
    }
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
pub trait NominalAgreementMarker: NominalNumberMarker {
    type Agreement: AgreementMarker;
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
pub trait RelativeTpGap: TpGap {}

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

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DynamicAgreement;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ThirdSingularAgreement;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ThirdPluralAgreement;

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

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContentForce;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RelativeForce;

#[doc(hidden)]
pub trait CpForce<G: TpGap>: private::Sealed + Copy {
    type Head: Copy;

    fn default_head() -> Self::Head;
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
pub(crate) enum DeterminerPhraseKind {
    BareNominal(Box<NounPhraseData>),
    DeterminedNominal {
        determiner: Determiner,
        nominal: Box<NounPhraseData>,
    },
    PossessedNominal {
        possessor: Box<DynamicDeterminerPhrase>,
        nominal: Box<NounPhraseData>,
    },
    ProperName(String),
    Pronoun {
        pronoun: Pronoun,
        reflexive: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeterminerPhrase<A: AgreementMarker = DynamicAgreement> {
    pub(crate) kind: DeterminerPhraseKind,
    _agreement: PhantomData<A>,
}

/// A determiner phrase whose agreement must be recovered from its surface form.
pub type DynamicDeterminerPhrase = DeterminerPhrase<DynamicAgreement>;

/// A determiner phrase that contributes third-person singular agreement.
pub type SingularDeterminerPhrase = DeterminerPhrase<ThirdSingularAgreement>;

/// A determiner phrase that contributes third-person plural agreement.
pub type PluralDeterminerPhrase = DeterminerPhrase<ThirdPluralAgreement>;

impl<A: AgreementMarker> DeterminerPhrase<A> {
    fn new(kind: DeterminerPhraseKind) -> Self {
        Self {
            kind,
            _agreement: PhantomData,
        }
    }

    pub(crate) fn erase(self) -> DynamicDeterminerPhrase {
        DeterminerPhrase::new(self.kind)
    }

    pub fn determiner_opt(&self) -> Option<Determiner> {
        match &self.kind {
            DeterminerPhraseKind::DeterminedNominal { determiner, .. } => Some(*determiner),
            DeterminerPhraseKind::BareNominal(_)
            | DeterminerPhraseKind::PossessedNominal { .. }
            | DeterminerPhraseKind::ProperName(_)
            | DeterminerPhraseKind::Pronoun { .. } => None,
        }
    }

    pub fn possessor_opt(&self) -> Option<&DynamicDeterminerPhrase> {
        match &self.kind {
            DeterminerPhraseKind::PossessedNominal { possessor, .. } => Some(possessor),
            DeterminerPhraseKind::BareNominal(_)
            | DeterminerPhraseKind::DeterminedNominal { .. }
            | DeterminerPhraseKind::ProperName(_)
            | DeterminerPhraseKind::Pronoun { .. } => None,
        }
    }

    pub fn proper_name_opt(&self) -> Option<&str> {
        match &self.kind {
            DeterminerPhraseKind::ProperName(name) => Some(name),
            DeterminerPhraseKind::BareNominal(_)
            | DeterminerPhraseKind::DeterminedNominal { .. }
            | DeterminerPhraseKind::PossessedNominal { .. }
            | DeterminerPhraseKind::Pronoun { .. } => None,
        }
    }

    pub fn pronoun_opt(&self) -> Option<Pronoun> {
        match &self.kind {
            DeterminerPhraseKind::Pronoun { pronoun, .. } => Some(*pronoun),
            DeterminerPhraseKind::BareNominal(_)
            | DeterminerPhraseKind::DeterminedNominal { .. }
            | DeterminerPhraseKind::PossessedNominal { .. }
            | DeterminerPhraseKind::ProperName(_) => None,
        }
    }

    pub fn is_reflexive(&self) -> bool {
        matches!(
            &self.kind,
            DeterminerPhraseKind::Pronoun {
                reflexive: true,
                ..
            }
        )
    }
}

impl SingularDeterminerPhrase {
    pub fn proper_name(name: impl Into<String>) -> Self {
        Self::new(DeterminerPhraseKind::ProperName(name.into()))
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

    fn determiner(self, determiner: Determiner) -> DeterminerPhrase<N::Agreement>
    where
        N: NominalAgreementMarker,
    {
        DeterminerPhrase::new(DeterminerPhraseKind::DeterminedNominal {
            determiner,
            nominal: self.nominal,
        })
    }

    pub fn the(self) -> DeterminerPhrase<N::Agreement>
    where
        N: NominalAgreementMarker,
    {
        self.determiner(Determiner::The)
    }

    pub fn possessor<P: Into<DynamicDeterminerPhrase>>(
        self,
        possessor: P,
    ) -> DeterminerPhrase<N::Agreement>
    where
        N: NominalAgreementMarker,
    {
        DeterminerPhrase::new(DeterminerPhraseKind::PossessedNominal {
            possessor: Box::new(possessor.into()),
            nominal: self.nominal,
        })
    }
}

impl<C: NominalCountabilityMarker> NominalDeterminerPhrase<SingularNumber, C> {
    pub fn this(self) -> SingularDeterminerPhrase {
        self.determiner(Determiner::This)
    }

    pub fn that(self) -> SingularDeterminerPhrase {
        self.determiner(Determiner::That)
    }
}

impl<C: NominalCountabilityMarker> NominalDeterminerPhrase<PluralNumber, C> {
    pub fn these(self) -> PluralDeterminerPhrase {
        self.determiner(Determiner::These)
    }

    pub fn those(self) -> PluralDeterminerPhrase {
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
    pub fn indefinite(self) -> SingularDeterminerPhrase {
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

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub enum VpArgumentSlot {
    Complement(VpComplement),
    GapObject,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase<G: PredicateGap = NoGap> {
    head: VerbEntry,
    arguments: Vec<VpArgumentSlot>,
    adjuncts: Vec<VpAdjunct>,
    _gap: PhantomData<G>,
}

impl VerbPhrase<NoGap> {
    pub fn new(head: impl Into<VerbEntry>) -> Self {
        Self {
            head: head.into(),
            arguments: Vec::new(),
            adjuncts: Vec::new(),
            _gap: PhantomData,
        }
    }

    pub fn object_gap(mut self) -> VerbPhrase<ObjectGap> {
        self.arguments.push(VpArgumentSlot::GapObject);
        VerbPhrase {
            head: self.head,
            arguments: self.arguments,
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
        self.arguments
            .push(VpArgumentSlot::Complement(complement.into()));
        self
    }

    pub fn adjunct<A: Into<VpAdjunct>>(mut self, adjunct: A) -> Self {
        self.adjuncts.push(adjunct.into());
        self
    }

    pub fn head(&self) -> &VerbEntry {
        &self.head
    }

    pub fn complements(&self) -> impl Iterator<Item = &VpComplement> + '_ {
        self.arguments.iter().filter_map(|slot| match slot {
            VpArgumentSlot::Complement(complement) => Some(complement),
            VpArgumentSlot::GapObject => None,
        })
    }

    pub fn has_object_gap(&self) -> bool {
        self.arguments
            .iter()
            .any(|slot| matches!(slot, VpArgumentSlot::GapObject))
    }

    #[doc(hidden)]
    pub fn argument_slots(&self) -> &[VpArgumentSlot] {
        &self.arguments
    }

    pub fn adjuncts(&self) -> &[VpAdjunct] {
        &self.adjuncts
    }
}

#[doc(hidden)]
pub trait SubjectLike: private::Sealed {
    type Agreement: AgreementMarker;

    fn into_subject(self) -> DynamicDeterminerPhrase;
}

#[derive(Clone)]
struct LittleVerbPhrase<G: TpGap = NoGap, A: AgreementMarker = DynamicAgreement> {
    subject: Option<DynamicDeterminerPhrase>,
    predicate: VerbPhrase<G::PredicateGap>,
    _agreement: PhantomData<A>,
}

impl<G: BaseTpGap> LittleVerbPhrase<G> {
    fn new(predicate: VerbPhrase<G>) -> Self {
        Self {
            subject: None,
            predicate,
            _agreement: PhantomData,
        }
    }
}

impl<G: TpGap, A: AgreementMarker> LittleVerbPhrase<G, A> {
    fn subject_opt(&self) -> Option<&DynamicDeterminerPhrase> {
        self.subject.as_ref()
    }

    fn predicate(&self) -> &VerbPhrase<G::PredicateGap> {
        &self.predicate
    }
}

impl<G: TpGap, A: AgreementMarker> std::fmt::Debug for LittleVerbPhrase<G, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LittleVerbPhrase")
            .field("subject", &self.subject)
            .field("predicate", &self.predicate)
            .finish()
    }
}

impl<G: TpGap, A: AgreementMarker, B: AgreementMarker> PartialEq<LittleVerbPhrase<G, B>>
    for LittleVerbPhrase<G, A>
{
    fn eq(&self, other: &LittleVerbPhrase<G, B>) -> bool {
        self.subject == other.subject && self.predicate == other.predicate
    }
}

impl<G: OvertTpGap, A: AgreementMarker> LittleVerbPhrase<G, A> {
    fn subject<S: SubjectLike>(self, subject: S) -> LittleVerbPhrase<G, S::Agreement> {
        LittleVerbPhrase {
            subject: Some(subject.into_subject()),
            predicate: self.predicate,
            _agreement: PhantomData,
        }
    }
}

impl<A: AgreementMarker> LittleVerbPhrase<NoGap, A> {
    fn subject_gap<N: NominalNumberMarker>(self) -> LittleVerbPhrase<SubjectGap<N>, A> {
        LittleVerbPhrase {
            subject: None,
            predicate: self.predicate,
            _agreement: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensePhrase<Form: TpForm, G: TpGap = NoGap, A: AgreementMarker = DynamicAgreement> {
    projection: LittleVerbPhrase<G, A>,
    form: Form,
    negative: bool,
}

impl<G: BaseTpGap> TensePhrase<BareInfinitive, G> {
    pub fn new(predicate: VerbPhrase<G>) -> Self {
        Self {
            projection: LittleVerbPhrase::new(predicate),
            form: BareInfinitive,
            negative: false,
        }
    }
}

impl<Form: TpForm, G: TpGap, A: AgreementMarker> TensePhrase<Form, G, A> {
    fn erase_agreement(self) -> TensePhrase<Form, G> {
        TensePhrase {
            projection: LittleVerbPhrase {
                subject: self.projection.subject,
                predicate: self.projection.predicate,
                _agreement: PhantomData,
            },
            form: self.form,
            negative: self.negative,
        }
    }

    fn map_form<Next: TpForm>(self, form: Next) -> TensePhrase<Next, G, A> {
        TensePhrase {
            projection: self.projection,
            form,
            negative: self.negative,
        }
    }

    pub fn present(self) -> TensePhrase<Finite, G, A> {
        self.map_form(Finite(Tense::Present))
    }

    pub fn past(self) -> TensePhrase<Finite, G, A> {
        self.map_form(Finite(Tense::Past))
    }

    pub fn bare_infinitive(self) -> TensePhrase<BareInfinitive, G, A> {
        self.map_form(BareInfinitive)
    }

    pub fn to_infinitive(self) -> TensePhrase<ToInfinitive, G, A> {
        self.map_form(ToInfinitive)
    }

    pub fn gerund_participle(self) -> TensePhrase<Gerund, G, A> {
        self.map_form(Gerund)
    }

    pub fn past_participle(self) -> TensePhrase<PastParticiple, G, A> {
        self.map_form(PastParticiple)
    }

    pub fn negative(mut self) -> Self {
        self.negative = true;
        self
    }

    pub fn subject_opt(&self) -> Option<&DynamicDeterminerPhrase> {
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

impl<Form: TpForm, G: OvertTpGap, A: AgreementMarker> TensePhrase<Form, G, A> {
    pub fn subject<S: SubjectLike>(self, subject: S) -> TensePhrase<Form, G, S::Agreement> {
        TensePhrase {
            projection: self.projection.subject(subject),
            form: self.form,
            negative: self.negative,
        }
    }
}

impl<Form: TpForm, A: AgreementMarker> TensePhrase<Form, NoGap, A> {
    pub fn subject_gap<N: NominalNumberMarker>(self) -> TensePhrase<Form, SubjectGap<N>, A> {
        TensePhrase {
            projection: self.projection.subject_gap(),
            form: self.form,
            negative: self.negative,
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct CpBuilder<G: TpGap = NoGap> {
    complement: TensePhrase<Finite, G>,
}

/// Builder returned by [`cp`] before choosing `content()` or `relative()`.
pub type ClauseBuilder<G = NoGap> = CpBuilder<G>;

/// A complementizer phrase introduced by an ordinary complementizer.
pub type ContentClause = ComplementizerPhrase<ContentForce, NoGap>;

/// A relative clause introduced by a relativizer such as `that`, `who`, or `which`.
pub type RelativeClause<G = NoGap> = ComplementizerPhrase<RelativeForce, G>;

impl<G: TpGap> CpBuilder<G> {
    fn new(complement: TensePhrase<Finite, G>) -> Self {
        Self { complement }
    }
}

impl CpBuilder<NoGap> {
    pub fn content(self) -> ContentClause {
        ComplementizerPhrase::new(self.complement)
    }
}

impl<G: RelativeTpGap> CpBuilder<G>
where
    RelativeForce: CpForce<G, Head = Relativizer>,
{
    pub fn relative(self) -> RelativeClause<G> {
        ComplementizerPhrase::new(self.complement)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplementizerPhrase<F: CpForce<G>, G: TpGap = NoGap> {
    head: F::Head,
    complement: Box<TensePhrase<Finite, G>>,
    _force: PhantomData<F>,
}

impl<F: CpForce<G>, G: TpGap> ComplementizerPhrase<F, G> {
    pub fn new(complement: TensePhrase<Finite, G>) -> Self {
        Self {
            head: F::default_head(),
            complement: Box::new(complement),
            _force: PhantomData,
        }
    }

    pub fn complement(&self) -> &TensePhrase<Finite, G> {
        &self.complement
    }
}

impl ComplementizerPhrase<ContentForce, NoGap> {
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

    pub fn head(&self) -> Complementizer {
        self.head
    }
}

#[doc(hidden)]
pub trait RelativeCpAttachment<N: NominalNumberMarker>: private::Sealed {
    fn into_np_adjunct(self) -> NpAdjunct;
}

impl<G: RelativeTpGap> ComplementizerPhrase<RelativeForce, G>
where
    RelativeForce: CpForce<G, Head = Relativizer>,
{
    pub fn relativizer(mut self, head: Relativizer) -> Self {
        self.head = head;
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
        self.head
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
    CP(ContentClause),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NpAdjunct {
    PP(PrepositionalPhrase),
    RelativeObject(RelativeClause<ObjectGap>),
    RelativeSubjectSingular(RelativeClause<SubjectGap<SingularNumber>>),
    RelativeSubjectPlural(RelativeClause<SubjectGap<PluralNumber>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApComplement {
    PP(PrepositionalPhrase),
    ToInf(TensePhrase<ToInfinitive>),
    CP(ContentClause),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdvpComplement {
    PP(PrepositionalPhrase),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PpComplement {
    DP(DynamicDeterminerPhrase),
    PP(PrepositionalPhrase),
    Gerund(TensePhrase<Gerund>),
    CP(ContentClause),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VpComplement {
    DP(DynamicDeterminerPhrase),
    PP(PrepositionalPhrase),
    AP(AdjectivePhrase),
    CP(ContentClause),
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
/// let phrase = cp(tp(vp("arrive")).past().subject(dp(Pronoun::She)))
///     .content()
///     .that();
/// assert_eq!(phrase.realize(), "that she arrived");
/// ```
///
/// ```compile_fail
/// use english_phrase::*;
///
/// let _ = cp(tp(vp("leave")).to_infinitive());
/// ```
pub fn cp<G: TpGap, A: AgreementMarker>(complement: TensePhrase<Finite, G, A>) -> ClauseBuilder<G> {
    CpBuilder::new(complement.erase_agreement())
}

#[doc(hidden)]
pub trait DpHead: private::Sealed {
    type Output;

    fn into_dp(self) -> Self::Output;
}

trait DpLike: private::Sealed {
    fn into_determiner_phrase(self) -> DynamicDeterminerPhrase;
}

impl<A: AgreementMarker> DpHead for DeterminerPhrase<A> {
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
    type Output = SingularDeterminerPhrase;

    fn into_dp(self) -> Self::Output {
        self.into()
    }
}

impl<A: AgreementMarker> DpLike for DeterminerPhrase<A> {
    fn into_determiner_phrase(self) -> DynamicDeterminerPhrase {
        self.erase()
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> DpLike for NominalDeterminerPhrase<N, C>
where
    N: NominalAgreementMarker,
{
    fn into_determiner_phrase(self) -> DynamicDeterminerPhrase {
        DeterminerPhrase::<N::Agreement>::from(self).erase()
    }
}

impl DpLike for PronominalDeterminerPhrase {
    fn into_determiner_phrase(self) -> DynamicDeterminerPhrase {
        self.into()
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> From<NominalDeterminerPhrase<N, C>>
    for DeterminerPhrase<N::Agreement>
where
    N: NominalAgreementMarker,
{
    fn from(value: NominalDeterminerPhrase<N, C>) -> Self {
        DeterminerPhrase::new(DeterminerPhraseKind::BareNominal(value.nominal))
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> From<NounPhrase<N, C>>
    for DeterminerPhrase<N::Agreement>
where
    N: NominalAgreementMarker,
{
    fn from(value: NounPhrase<N, C>) -> Self {
        NominalDeterminerPhrase::new(value).into()
    }
}

impl From<PronominalDeterminerPhrase> for DynamicDeterminerPhrase {
    fn from(value: PronominalDeterminerPhrase) -> Self {
        DeterminerPhrase::new(DeterminerPhraseKind::Pronoun {
            pronoun: value.pronoun,
            reflexive: value.reflexive,
        })
    }
}

impl From<Pronoun> for DynamicDeterminerPhrase {
    fn from(value: Pronoun) -> Self {
        PronominalDeterminerPhrase::new(value).into()
    }
}

impl From<Name> for SingularDeterminerPhrase {
    fn from(value: Name) -> Self {
        DeterminerPhrase::proper_name(value.0)
    }
}

impl From<SingularDeterminerPhrase> for DynamicDeterminerPhrase {
    fn from(value: SingularDeterminerPhrase) -> Self {
        value.erase()
    }
}

impl From<PluralDeterminerPhrase> for DynamicDeterminerPhrase {
    fn from(value: PluralDeterminerPhrase) -> Self {
        value.erase()
    }
}

impl<A: AgreementMarker> SubjectLike for DeterminerPhrase<A> {
    type Agreement = A;

    fn into_subject(self) -> DynamicDeterminerPhrase {
        self.erase()
    }
}

impl SubjectLike for PronominalDeterminerPhrase {
    type Agreement = DynamicAgreement;

    fn into_subject(self) -> DynamicDeterminerPhrase {
        self.into()
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> SubjectLike
    for NominalDeterminerPhrase<N, C>
where
    N: NominalAgreementMarker,
{
    type Agreement = N::Agreement;

    fn into_subject(self) -> DynamicDeterminerPhrase {
        DeterminerPhrase::<N::Agreement>::from(self).erase()
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

impl From<ContentClause> for NpComplement {
    fn from(value: ContentClause) -> Self {
        Self::CP(value)
    }
}

impl From<PrepositionalPhrase> for NpAdjunct {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl<N: NominalNumberMarker> RelativeCpAttachment<N> for RelativeClause<ObjectGap> {
    fn into_np_adjunct(self) -> NpAdjunct {
        NpAdjunct::RelativeObject(self)
    }
}

impl RelativeCpAttachment<SingularNumber> for RelativeClause<SubjectGap<SingularNumber>> {
    fn into_np_adjunct(self) -> NpAdjunct {
        NpAdjunct::RelativeSubjectSingular(self)
    }
}

impl RelativeCpAttachment<PluralNumber> for RelativeClause<SubjectGap<PluralNumber>> {
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

impl From<ContentClause> for ApComplement {
    fn from(value: ContentClause) -> Self {
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

impl From<ContentClause> for PpComplement {
    fn from(value: ContentClause) -> Self {
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

impl From<ContentClause> for VpComplement {
    fn from(value: ContentClause) -> Self {
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
impl private::Sealed for DynamicAgreement {}
impl private::Sealed for ThirdSingularAgreement {}
impl private::Sealed for ThirdPluralAgreement {}
impl private::Sealed for NoGap {}
impl private::Sealed for ObjectGap {}
impl<N: NominalNumberMarker> private::Sealed for SubjectGap<N> {}
impl private::Sealed for ContentForce {}
impl private::Sealed for RelativeForce {}
impl<A: AgreementMarker> private::Sealed for DeterminerPhrase<A> {}
impl<N: NominalNumberMarker, C: NominalCountabilityMarker> private::Sealed
    for NominalDeterminerPhrase<N, C>
{
}
impl private::Sealed for PronominalDeterminerPhrase {}
impl<N: NominalNumberMarker, C: NominalCountabilityMarker> private::Sealed for NounPhrase<N, C> {}
impl<G: PredicateGap> private::Sealed for VerbPhrase<G> {}
impl private::Sealed for VpArgumentSlot {}
impl<Form: TpForm, G: TpGap, A: AgreementMarker> private::Sealed for TensePhrase<Form, G, A> {}
impl<F: CpForce<G>, G: TpGap> private::Sealed for ComplementizerPhrase<F, G> {}
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

impl NominalAgreementMarker for SingularNumber {
    type Agreement = ThirdSingularAgreement;
}

impl NominalAgreementMarker for PluralNumber {
    type Agreement = ThirdPluralAgreement;
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

impl AgreementMarker for DynamicAgreement {}

impl AgreementMarker for ThirdSingularAgreement {
    fn agreement() -> Option<(Person, Number)> {
        Some((Person::Third, Number::Singular))
    }
}

impl AgreementMarker for ThirdPluralAgreement {
    fn agreement() -> Option<(Person, Number)> {
        Some((Person::Third, Number::Plural))
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

impl CpForce<NoGap> for ContentForce {
    type Head = Complementizer;

    fn default_head() -> Self::Head {
        Complementizer::Null
    }
}

impl CpForce<ObjectGap> for RelativeForce {
    type Head = Relativizer;

    fn default_head() -> Self::Head {
        Relativizer::Null
    }
}

impl<N: NominalNumberMarker> CpForce<SubjectGap<N>> for RelativeForce {
    type Head = Relativizer;

    fn default_head() -> Self::Head {
        Relativizer::Null
    }
}

impl RelativeTpGap for ObjectGap {}
impl<N: NominalNumberMarker> RelativeTpGap for SubjectGap<N> {}
