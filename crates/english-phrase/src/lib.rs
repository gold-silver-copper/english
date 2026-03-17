#![doc = include_str!("../README.md")]

pub mod lexical;
pub mod realization;
pub mod syntax;

pub use english::{Adj, Gender, Noun, Number, Person, Verb};

pub use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
pub use crate::realization::{Realizable, RealizationOptions, Terminal};
pub use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, AdvpComplement, ApComplement, BareInfinitive, ClauseBuilder,
    ComplementizerPhrase, ContentClause, CountNoun, Countability, DeterminerPhrase,
    DynamicDeterminerPhrase, Finite, Gerund, MassNoun, Name, NoGap, NominalDeterminerPhrase,
    NounPhrase, NpAdjunct, NpComplement, NpModifier, ObjectGap, PastParticiple,
    PluralDeterminerPhrase, PluralNumber, PpComplement, PrepositionalPhrase,
    PronominalDeterminerPhrase, RelativeClause, SingularDeterminerPhrase, SingularNumber,
    SubjectGap, Tense, TensePhrase, ToInfinitive, UnknownCountability, VerbForm, VerbPhrase,
    VpAdjunct, VpComplement, adjp, advp, cp, dp, name, np, pp, tp, vp,
};
