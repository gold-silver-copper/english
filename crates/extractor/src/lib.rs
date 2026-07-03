//! Wiktionary dump -> deterministic lookup tables, in five stages:
//!
//! 1. [`args`]/[`pipeline`] — CLI and orchestration (called via `cargo xtask
//!    refresh-data`); decides what runs and, critically, what ABORTS a refresh
//!    before anything is written.
//! 2. [`extract`] — scans the filtered dump and turns entries into per-lemma
//!    [`registry::Candidate`]s; this module owns every editorial POLICY
//!    (which forms are admissible, which pattern is canonical, what reserves
//!    the bare key).
//! 3. [`registry`] — the determinism core: resolves candidates against the
//!    checked-in assignment lockfiles so published sense keys never change
//!    meaning; owns the immutability guard `check-registry` enforces in CI.
//! 4. [`bootstrap`]/[`file_generation`] — the single lock -> PHF emission
//!    path, byte-deterministic because CI byte-compares its output.
//! 5. [`checks`] — the dump-driven accuracy harness (`cargo xtask accuracy`).
//!
//! The division of labor is deliberate: `extract` may change policy freely
//! (what SHOULD be published), while `registry` guarantees stability (already
//! published keys keep their meaning regardless of policy churn).

use std::error::Error;

pub mod args;
pub mod bootstrap;
pub mod checks;
pub mod extract;
pub mod file_generation;
pub mod helpers;
pub mod pipeline;
pub mod registry;

pub use args::Config;

pub fn run_from_env() -> Result<(), Box<dyn Error>> {
    let config = args::parse_args()?;
    pipeline::run(&config)
}
