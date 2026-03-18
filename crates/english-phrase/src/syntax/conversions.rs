use super::ast::{
    AdjectivePhrase, AdverbPhrase, AdvpComplement, ApComplement, ComplementizerPhrase,
    ContentClause, DeterminerPhrase, DeterminerPhraseKind, DynamicDeterminerPhrase, Name,
    NominalDeterminerPhrase, NounPhrase, NpAdjunct, NpComplement, NpModifier, PpComplement,
    PrepositionalPhrase, PronominalDeterminerPhrase, RelativeClause, TensePhrase, VerbPhrase,
    VpAdjunct, VpArgumentSlot, VpComplement,
};
use super::features::{
    AgreementMarker, BareInfinitive, DynamicAgreement, Gerund, NominalAgreementMarker,
    NominalCountabilityMarker, NominalNumberMarker, ObjectGap, PastParticiple, PluralNumber,
    PredicateGap, SingularNumber, SubjectGap, ToInfinitive, TpForm, TpGap,
};
use super::private;
use crate::lexical::Pronoun;

#[doc(hidden)]
pub trait IntoDynamicDeterminerPhrase: private::Sealed {
    fn into_dynamic_dp(self) -> DynamicDeterminerPhrase;
}

#[doc(hidden)]
pub trait SubjectLike: private::Sealed + IntoDynamicDeterminerPhrase {
    type Agreement: AgreementMarker;
}

#[doc(hidden)]
pub trait RelativeCpAttachment<N: NominalNumberMarker>: private::Sealed {
    fn into_np_adjunct(self) -> NpAdjunct;
}

impl<A: AgreementMarker> IntoDynamicDeterminerPhrase for DeterminerPhrase<A> {
    fn into_dynamic_dp(self) -> DynamicDeterminerPhrase {
        self.erase()
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> IntoDynamicDeterminerPhrase
    for NominalDeterminerPhrase<N, C>
where
    N: NominalAgreementMarker,
{
    fn into_dynamic_dp(self) -> DynamicDeterminerPhrase {
        DeterminerPhrase::<N::Agreement>::from(self).erase()
    }
}

impl IntoDynamicDeterminerPhrase for PronominalDeterminerPhrase {
    fn into_dynamic_dp(self) -> DynamicDeterminerPhrase {
        self.into()
    }
}

impl SubjectLike for PronominalDeterminerPhrase {
    type Agreement = DynamicAgreement;
}

impl<A: AgreementMarker> SubjectLike for DeterminerPhrase<A> {
    type Agreement = A;
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> SubjectLike
    for NominalDeterminerPhrase<N, C>
where
    N: NominalAgreementMarker,
{
    type Agreement = N::Agreement;
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

impl From<Name> for super::ast::SingularDeterminerPhrase {
    fn from(value: Name) -> Self {
        DeterminerPhrase::proper_name(value.0)
    }
}

impl From<super::ast::SingularDeterminerPhrase> for DynamicDeterminerPhrase {
    fn from(value: super::ast::SingularDeterminerPhrase) -> Self {
        value.erase()
    }
}

impl From<super::ast::PluralDeterminerPhrase> for DynamicDeterminerPhrase {
    fn from(value: super::ast::PluralDeterminerPhrase) -> Self {
        value.erase()
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

impl<T: IntoDynamicDeterminerPhrase> From<T> for PpComplement {
    fn from(value: T) -> Self {
        Self::DP(value.into_dynamic_dp())
    }
}

impl From<PrepositionalPhrase> for PpComplement {
    fn from(value: PrepositionalPhrase) -> Self {
        Self::PP(value)
    }
}

impl From<TensePhrase<super::features::Gerund>> for PpComplement {
    fn from(value: TensePhrase<super::features::Gerund>) -> Self {
        Self::Gerund(value)
    }
}

impl From<ContentClause> for PpComplement {
    fn from(value: ContentClause) -> Self {
        Self::CP(value)
    }
}

impl<T: IntoDynamicDeterminerPhrase> From<T> for VpComplement {
    fn from(value: T) -> Self {
        Self::DP(value.into_dynamic_dp())
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
impl<F: super::features::CpForce<G>, G: TpGap> private::Sealed for ComplementizerPhrase<F, G> {}
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
