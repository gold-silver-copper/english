mod error;

pub mod lexical;
pub mod realization;
pub mod syntax;

pub use english::{Adj, Noun, Number, Person, Verb};

pub use crate::error::{RealizationError, RealizationResult};
pub use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
pub use crate::realization::{Realizable, RealizationOptions, Terminal};
pub use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, AdvpComplement, AdvpModifier, ApComplement, ApModifier,
    DeterminerPhrase, Name, NominalDeterminerPhrase, NounPhrase, NpComplement, NpModifier, Phrase,
    PpComplement, PrepositionalPhrase, PronominalDeterminerPhrase, Tense, TensePhrase, VerbForm,
    VerbPhrase, VpAdjunct, VpComplement, adjp, advp, dp, name, np, pp, tp, vp,
};
