use english::{Animacy, Gender, Number, Person};

use crate::cat::Cat;
use crate::derivation::AgreementInfo;

/// A user-supplied lexical entry.
///
/// `english-ccg` does not ship a built-in lexicon. Users provide the surface
/// form, syntactic category, and any nominal features needed for agreement or
/// pronoun realization.
#[derive(Debug, Clone)]
pub struct LexEntry {
    surface: String,
    cat: Cat,
    animacy: Option<Animacy>,
    agreement: Option<AgreementInfo>,
}

/// Construct a lexical entry from a surface form and a category.
pub fn entry(surface: impl Into<String>, cat: Cat) -> LexEntry {
    LexEntry::new(surface, cat)
}

impl LexEntry {
    /// Create a lexical entry from a surface form and a category.
    pub fn new(surface: impl Into<String>, cat: Cat) -> Self {
        Self {
            surface: surface.into(),
            cat,
            animacy: None,
            agreement: None,
        }
    }

    /// The lexical surface form supplied by the user.
    pub fn surface(&self) -> &str {
        &self.surface
    }

    /// The lexical category supplied by the user.
    pub fn cat(&self) -> &Cat {
        &self.cat
    }

    pub(crate) fn animacy(&self) -> Option<Animacy> {
        self.animacy
    }

    pub(crate) fn agreement(&self) -> Option<AgreementInfo> {
        self.agreement
    }

    /// Mark the entry as animate.
    pub fn animate(mut self) -> Self {
        self.animacy = Some(Animacy::Animate);
        self
    }

    /// Mark the entry as inanimate.
    pub fn inanimate(mut self) -> Self {
        self.animacy = Some(Animacy::Inanimate);
        self
    }

    /// Attach agreement features used by subject agreement and pronoun
    /// realization.
    pub fn with_agreement(mut self, person: Person, number: Number, gender: Gender) -> Self {
        self.agreement = Some(AgreementInfo {
            person,
            number,
            gender,
        });
        self
    }
}
