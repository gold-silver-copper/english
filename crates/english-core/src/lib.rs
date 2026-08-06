//! The dependency-free REGULAR fallback / prediction engine for English inflection.
//!
//! `english-core` knows nothing about Wiktionary or lookup tables: it encodes a
//! compact approximation of productive English morphology (add `-s`, `-ed`,
//! `-er`, with spelling adjustments) plus a few suffix-class lists that
//! generalize over endings and compounds. It is intentionally small and is not a
//! guarantee of correct inflection for arbitrary out-of-vocabulary words; the
//! table-backed `english` crate is the correctness-oriented public API.
//!
//! # The contract with the `english` crate (do not break casually)
//!
//! These rules serve two roles at once:
//! 1. the runtime FALLBACK for any word the generated tables don't list;
//! 2. the extractor's PREDICTION: a Wiktionary attestation equal to the rule
//!    output is dropped at table-generation time (the rule will produce it),
//!    and its presence reserves the bare key for the rule engine.
//!
//! Consequently, changing any rule here changes what counts as "irregular" and
//! REQUIRES regenerating the tables (`cargo xtask refresh-data`). Two dump-free
//! tests partially guard the drift — the `english` crate's `rule_table_sync`
//! test and this crate's `regular_rules_golden` test — but `cargo xtask accuracy`
//! (with the dump) is the authoritative check, and also measures rule quality
//! against every plain attested form.
//!
//! Inputs are assumed to be lowercase lemmas; case handling is the `english`
//! crate's responsibility.

mod adj;
mod adverb;
pub mod grammar;
mod noun;
pub mod sense_key;
mod utils;
mod verb;
pub use crate::grammar::*;

/// Namespace struct for the rule engine — all functionality is associated
/// functions (`EnglishCore::noun`, `::verb`, `::comparative`, ...); there is no
/// state to construct.
pub struct EnglishCore {}
