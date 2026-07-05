//! Wiktionary dump -> deterministic lookup tables, in four stages:
//!
//! 1. [`args`]/[`pipeline`] — CLI and orchestration (called via `cargo xtask
//!    refresh-data`): filter the dump, extract, and emit the tables.
//! 2. [`extract`] — scans the filtered dump and turns entries into per-lemma
//!    [`assign::Candidate`]s; this module owns every editorial POLICY (which
//!    forms are admissible, which pattern is canonical, what reserves the bare
//!    key).
//! 3. [`assign`] — numbers each lemma's candidates into deterministic `_<n>`
//!    keys by a pure sort of their emitted forms. No lockfile, no identity, no
//!    carry-forward: the output is a function of the current dump alone.
//! 4. [`bootstrap`]/[`file_generation`] — the single candidate -> PHF emission
//!    path; [`checks`] is the dump-driven accuracy harness (`cargo xtask
//!    accuracy`).
//!
//! The division of labor is deliberate: `extract` owns policy (what SHOULD be
//! published) and `assign` owns numbering (a reproducible sort). Neither keeps
//! state between runs — a refresh regenerates everything from the dump.

use std::error::Error;

pub mod args;
pub mod assign;
pub mod bootstrap;
pub mod checks;
pub mod extract;
pub mod file_generation;
pub mod helpers;
pub mod pipeline;

pub use args::Config;

pub fn run_from_env() -> Result<(), Box<dyn Error>> {
    let config = args::parse_args()?;
    pipeline::run(&config)
}
