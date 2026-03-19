use super::private;
use english::{Number, Person};
use std::marker::PhantomData;

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
pub struct Finite(pub(crate) Tense);

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
