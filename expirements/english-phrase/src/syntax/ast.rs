use super::features::{
    AgreementMarker, BareInfinitive, ContentForce, Countability, CpForce, DynamicAgreement, Finite,
    Gerund, NoGap, NominalCountabilityMarker, NominalNumberMarker, ObjectGap, PastParticiple,
    PluralNumber, PredicateGap, RelativeForce, SingularNumber, SubjectGap, ThirdPluralAgreement,
    ThirdSingularAgreement, ToInfinitive, TpForm, TpGap, UnknownCountability,
};
use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
use english::Number;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(pub(crate) String);

impl Name {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
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
    pub(crate) _marker: PhantomData<(N, C)>,
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
    pub(crate) _agreement: PhantomData<A>,
}

/// A determiner phrase whose agreement must be recovered from its surface form.
pub type DynamicDeterminerPhrase = DeterminerPhrase<DynamicAgreement>;

/// A determiner phrase that contributes third-person singular agreement.
pub type SingularDeterminerPhrase = DeterminerPhrase<ThirdSingularAgreement>;

/// A determiner phrase that contributes third-person plural agreement.
pub type PluralDeterminerPhrase = DeterminerPhrase<ThirdPluralAgreement>;

impl<A: AgreementMarker> DeterminerPhrase<A> {
    pub(crate) fn new(kind: DeterminerPhraseKind) -> Self {
        Self {
            kind,
            _agreement: PhantomData,
        }
    }

    pub(crate) fn erase(self) -> DynamicDeterminerPhrase {
        DeterminerPhrase::new(self.kind)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NominalDeterminerPhrase<
    N: NominalNumberMarker = SingularNumber,
    C: NominalCountabilityMarker = UnknownCountability,
> {
    pub(crate) nominal: Box<NounPhraseData>,
    pub(crate) _marker: PhantomData<(N, C)>,
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> NominalDeterminerPhrase<N, C> {
    pub(crate) fn new(nominal: NounPhrase<N, C>) -> Self {
        Self {
            nominal: Box::new(nominal.data),
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PronominalDeterminerPhrase {
    pub(crate) pronoun: Pronoun,
    pub(crate) reflexive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdjectivePhrase {
    pub(crate) modifier: Option<Box<AdverbPhrase>>,
    pub(crate) head: AdjectiveEntry,
    pub(crate) complements: Vec<ApComplement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdverbPhrase {
    pub(crate) modifier: Option<Box<AdverbPhrase>>,
    pub(crate) head: AdverbEntry,
    pub(crate) complements: Vec<AdvpComplement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrepositionalPhrase {
    pub(crate) head: PrepositionEntry,
    pub(crate) complement: Box<PpComplement>,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub enum VpArgumentSlot {
    Complement(VpComplement),
    GapObject,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbPhrase<G: PredicateGap = NoGap> {
    pub(crate) head: VerbEntry,
    pub(crate) arguments: Vec<VpArgumentSlot>,
    pub(crate) adjuncts: Vec<VpAdjunct>,
    pub(crate) _gap: PhantomData<G>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensePhrase<Form: TpForm, G: TpGap = NoGap, A: AgreementMarker = DynamicAgreement> {
    pub(crate) subject: Option<DynamicDeterminerPhrase>,
    pub(crate) predicate: VerbPhrase<G::PredicateGap>,
    pub(crate) form: Form,
    pub(crate) negative: bool,
    pub(crate) _agreement: PhantomData<A>,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct CpBuilder<G: TpGap = NoGap> {
    pub(crate) complement: TensePhrase<Finite, G>,
}

/// Builder returned by [`crate::syntax::cp`] before choosing `content()` or `relative()`.
pub type ClauseBuilder<G = NoGap> = CpBuilder<G>;

/// A complementizer phrase introduced by an ordinary complementizer.
pub type ContentClause = ComplementizerPhrase<ContentForce, NoGap>;

/// A relative clause introduced by a relativizer such as `that`, `who`, or `which`.
pub type RelativeClause<G = NoGap> = ComplementizerPhrase<RelativeForce, G>;

#[derive(Debug, Clone, PartialEq)]
pub struct ComplementizerPhrase<F: CpForce<G>, G: TpGap = NoGap> {
    pub(crate) head: F::Head,
    pub(crate) complement: Box<TensePhrase<Finite, G>>,
    pub(crate) _force: PhantomData<F>,
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
