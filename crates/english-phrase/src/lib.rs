pub mod lexical;
pub mod realization;
pub mod syntax;

pub use english::{Adj, Noun, Number, Person, Verb};

pub use crate::lexical::{
    AdjectiveEntry, AdverbEntry, Determiner, NounEntry, PrepositionEntry, Pronoun, VerbEntry,
};
pub use crate::realization::{
    RealizationError, RealizationResult, realize_adjective_phrase, realize_adverb_phrase,
    realize_clause, realize_determiner_phrase, realize_phrase, realize_prepositional_phrase,
    realize_sentence, realize_verb_phrase,
};
pub use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, DeterminerHead, DeterminerPhrase, Phrase, PrepositionalPhrase,
    Tense, VerbForm, VerbPhrase, adjp, advp, dp, pp, pronoun_dp, proper_name, vp,
};
