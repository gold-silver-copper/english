mod private {
    pub trait Sealed {}
}

mod ast;
mod builders;
mod conversions;
mod features;

pub use ast::{
    AdjectivePhrase, AdverbPhrase, AdvpComplement, ApComplement, ClauseBuilder,
    ComplementizerPhrase, ContentClause, DeterminerPhrase, DynamicDeterminerPhrase, Name,
    NominalDeterminerPhrase, NounPhrase, NpAdjunct, NpComplement, NpModifier,
    PluralDeterminerPhrase, PpComplement, PrepositionalPhrase, PronominalDeterminerPhrase,
    RelativeClause, SingularDeterminerPhrase, TensePhrase, VerbPhrase, VpAdjunct, VpComplement,
};
#[doc(hidden)]
pub use builders::DpHead;
pub use builders::{adjp, advp, cp, dp, name, np, pp, tp, vp};
#[doc(hidden)]
pub use conversions::IntoDynamicDeterminerPhrase;
#[doc(hidden)]
pub use conversions::{RelativeCpAttachment, SubjectLike};
#[doc(hidden)]
pub use features::{
    AgreementMarker, BareInfinitive, BaseTpGap, Complementizer, ContentForce, CountNoun,
    Countability, CpForce, DynamicAgreement, Finite, Gerund, MassNoun, NoGap,
    NominalAgreementMarker, NominalCountabilityMarker, NominalNumberMarker, NonfiniteTpForm,
    ObjectGap, OvertTpGap, PastParticiple, PluralNumber, PredicateGap, RelativeForce,
    RelativeTpGap, Relativizer, SingularNumber, SubjectGap, Tense, ThirdPluralAgreement,
    ThirdSingularAgreement, ToInfinitive, TpForm, TpGap, UnknownCountability, VerbForm,
};

pub(crate) use ast::{DeterminerPhraseKind, NounPhraseData, VpArgumentSlot};
