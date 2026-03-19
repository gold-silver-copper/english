//! A CCG-native English sentence generator built around lexical categories and
//! a fixed set of combinatory rules.
//!
//! `english-ccg` stays close to formal CCG: users provide lexical entries,
//! combine them with `+`, and inspect or realize the resulting derivation.
//! There are no phrase builders such as `tp()`, `vp()`, or `np()`. Structure
//! emerges from categories and combinators alone.
//!
//! # Category Notation
//!
//! Categories use standard CCG slash notation plus a small family of explicit
//! nonfinite VP categories:
//!
//! - `S`, `NP`, `N`, `PP`
//! - `VP[bare]`
//! - `VP[to]`
//! - `VP[prespart]`
//! - `VP[pastpart]`
//!
//! Use the raw-string `cat!` macro when constructing categories inline:
//!
//! ```rust
//! use english_ccg::{cat, Cat, VpForm};
//!
//! assert_eq!(cat!(r"(S\NP)/NP"), Cat::Fwd(Box::new(cat!(r"S\NP")), Box::new(Cat::NP)));
//! assert_eq!(cat!(r"VP[to]"), Cat::VP(VpForm::To));
//! ```
//!
//! # Auxiliary Naming
//!
//! Auxiliary terminal methods follow a deliberately explicit naming scheme:
//!
//! `<selected-complement-form>_<auxiliary-function>()`
//!
//! For example:
//!
//! - `bare_infinitive_modal()`
//! - `to_infinitive_modal()`
//! - `past_participle_perfect()`
//! - `present_participle_progressive()`
//! - `past_participle_passive()`
//! - `bare_infinitive_support()`
//!
//! The first component states which category the auxiliary selects; the second
//! states the construction it contributes.
//!
//! # Formal Consequences
//!
//! The current grammar treats nonfinite verb forms as genuine categories, not
//! as mere realization hints:
//!
//! - `verb(...).bare()` yields `VP[bare]` or `VP[bare]/NP`
//! - `verb(...).progressive()` yields `VP[prespart]` or `VP[prespart]/NP`
//! - `verb(...).perfective()` yields `VP[pastpart]` or `VP[pastpart]/NP`
//! - `inf()` is the lexical item `to : VP[to]/VP[bare]`
//!
//! That means auxiliary selection is enforced by the derivation itself. For
//! example, `ought` requires `VP[to]`, so `ought + bare VP` fails unless `to`
//! intervenes.
//!
//! ```rust
//! use english::Animacy;
//! use english_ccg::prelude::*;
//!
//! let alice = proper("Alice");
//! let repair = tv("repair");
//! let bridge = common("bridge", Animacy::Inanimate);
//!
//! let s = name(&alice)
//!     + aux("ought").invariant().to_infinitive_modal()
//!     + inf()
//!     + verb(&repair).bare()
//!     + (det("the") + noun(&bridge));
//!
//! assert_eq!(
//!     realize_as(&s, RealizeOpts::sentence()),
//!     "Alice ought to repair the bridge."
//! );
//! ```
//!
//! # Lexical Workflow
//!
//! The intended workflow is:
//!
//! 1. Create lexical entries with [`entry`] or the helper constructors.
//! 2. Turn them into CCG items with [`name`], [`noun`], [`verb`], or [`pro`].
//! 3. Combine them with `+`.
//! 4. Realize the result with [`realize`] or [`realize_as`].
//!
//! Invalid combinations fail with category diagnostics that report the
//! mismatched spans, categories, and attempted combinatory rules.

pub mod builders;
pub mod cat;
pub mod combinators;
pub mod derivation;
pub mod helpers;
pub mod lexicon;
pub mod morphology;
pub mod realization;

pub use builders::*;
pub use cat::*;
pub use combinators::*;
pub use derivation::*;
pub use helpers::*;
pub use lexicon::*;
pub use morphology::*;
pub use realization::*;

pub mod prelude {
    pub use crate::builders::*;
    pub use crate::cat::*;
    pub use crate::combinators::*;
    pub use crate::derivation::*;
    pub use crate::helpers::*;
    pub use crate::lexicon::*;
    pub use crate::morphology::*;
    pub use crate::realization::*;
}
