use super::ast::{
    AdjectivePhrase, AdverbPhrase, AdvpComplement, ApComplement, ClauseBuilder,
    ComplementizerPhrase, ContentClause, CpBuilder, DeterminerPhrase, DeterminerPhraseKind,
    DynamicDeterminerPhrase, Name, NominalDeterminerPhrase, NounPhrase, NounPhraseData, NpAdjunct,
    NpComplement, NpModifier, PluralDeterminerPhrase, PpComplement, PrepositionalPhrase,
    PronominalDeterminerPhrase, RelativeClause, SingularDeterminerPhrase, TensePhrase, VerbPhrase,
    VpAdjunct, VpArgumentSlot, VpComplement,
};
use super::conversions::{IntoDynamicDeterminerPhrase, RelativeCpAttachment, SubjectLike};
use super::features::{
    AgreementMarker, BareInfinitive, Complementizer, ContentForce, CountNoun, Countability,
    CpForce, Finite, Gerund, NoGap, NominalAgreementMarker, NominalCountabilityMarker,
    NominalNumberMarker, ObjectGap, OvertTpGap, PastParticiple, PluralNumber, PredicateGap,
    RelativeForce, RelativeTpGap, Relativizer, SingularNumber, SubjectGap, Tense, ToInfinitive,
    TpForm, TpGap, UnknownCountability,
};
use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
use english::Number;
use std::marker::PhantomData;

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

    pub fn mass(self) -> NounPhrase<N, super::features::MassNoun> {
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

impl<A: AgreementMarker> DeterminerPhrase<A> {
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

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> NominalDeterminerPhrase<N, C> {
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

    pub fn possessor<P: IntoDynamicDeterminerPhrase>(
        self,
        possessor: P,
    ) -> DeterminerPhrase<N::Agreement>
    where
        N: NominalAgreementMarker,
    {
        DeterminerPhrase::new(DeterminerPhraseKind::PossessedNominal {
            possessor: Box::new(possessor.into_dynamic_dp()),
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

    pub fn adjuncts(&self) -> &[VpAdjunct] {
        &self.adjuncts
    }

    #[doc(hidden)]
    pub fn argument_slots(&self) -> &[VpArgumentSlot] {
        &self.arguments
    }
}

impl<G: super::features::BaseTpGap> TensePhrase<BareInfinitive, G> {
    pub fn new(predicate: VerbPhrase<G>) -> Self {
        Self {
            subject: None,
            predicate,
            form: BareInfinitive,
            negative: false,
            _agreement: PhantomData,
        }
    }
}

impl<Form: TpForm, G: TpGap, A: AgreementMarker> TensePhrase<Form, G, A> {
    pub(crate) fn erase_agreement(self) -> TensePhrase<Form, G> {
        TensePhrase {
            subject: self.subject,
            predicate: self.predicate,
            form: self.form,
            negative: self.negative,
            _agreement: PhantomData,
        }
    }

    fn map_form<Next: TpForm>(self, form: Next) -> TensePhrase<Next, G, A> {
        TensePhrase {
            subject: self.subject,
            predicate: self.predicate,
            form,
            negative: self.negative,
            _agreement: PhantomData,
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
        self.subject.as_ref()
    }

    pub fn form(&self) -> super::features::VerbForm {
        self.form.verb_form()
    }

    pub fn is_negative(&self) -> bool {
        self.negative
    }

    pub fn predicate(&self) -> &VerbPhrase<G::PredicateGap> {
        &self.predicate
    }
}

impl<Form: TpForm, G: OvertTpGap, A: AgreementMarker> TensePhrase<Form, G, A> {
    pub fn subject<S: SubjectLike>(self, subject: S) -> TensePhrase<Form, G, S::Agreement> {
        TensePhrase {
            subject: Some(subject.into_dynamic_dp()),
            predicate: self.predicate,
            form: self.form,
            negative: self.negative,
            _agreement: PhantomData,
        }
    }
}

impl<Form: TpForm, A: AgreementMarker> TensePhrase<Form, NoGap, A> {
    pub fn subject_gap<N: NominalNumberMarker>(self) -> TensePhrase<Form, SubjectGap<N>, A> {
        TensePhrase {
            subject: None,
            predicate: self.predicate,
            form: self.form,
            negative: self.negative,
            _agreement: PhantomData,
        }
    }
}

impl<G: TpGap> CpBuilder<G> {
    pub(crate) fn new(complement: TensePhrase<Finite, G>) -> Self {
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
    G: super::features::BaseTpGap,
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
pub trait DpHead: super::private::Sealed {
    type Output;

    fn into_dp(self) -> Self::Output;
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
