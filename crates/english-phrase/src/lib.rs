mod desugar;
mod error;
mod internal;

pub mod lexical;
pub mod realization;
pub mod syntax;

pub use english::{Adj, Noun, Number, Person, Verb};

pub use crate::error::{RealizationError, RealizationResult};
pub use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
pub use crate::realization::{
    Realizable, realize_adjective_phrase, realize_adverb_phrase, realize_determiner_phrase,
    realize_noun_phrase, realize_phrase, realize_prepositional_phrase, realize_sentence,
    realize_tense_phrase, realize_verb_phrase,
};
pub use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, AdvpComplement, AdvpModifier, ApComplement, ApModifier,
    DeterminerHead, DeterminerPhrase, Name, NounPhrase, NpComplement, NpModifier, Phrase,
    PpComplement, PrepositionalPhrase, Sentence, Tense, TensePhrase, Terminal, VerbForm,
    VerbPhrase, VpAdjunct, VpComplement, adjp, advp, dp, name, np, pp, tp, vp,
};
