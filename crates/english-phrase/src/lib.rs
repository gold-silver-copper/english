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
    AdjectivePhrase, AdverbPhrase, Complementizer, ComplementizerPhrase, DeterminerPhrase, Name,
    NominalDeterminerPhrase, NounPhrase, Phrase, PrepositionalPhrase, PronominalDeterminerPhrase,
    Tense, TensePhrase, VerbForm, VerbPhrase, adjp, advp, cp, dp, name, np, pp, tp, vp,
};
