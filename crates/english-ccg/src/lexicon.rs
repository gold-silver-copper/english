use english::{Animacy, Gender, Number, Person};

use crate::cat::Cat;
use crate::derivation::AgreementInfo;

#[derive(Debug, Clone)]
pub struct LexEntry {
    surface: String,
    cat: Cat,
    animacy: Option<Animacy>,
    agreement: Option<AgreementInfo>,
}

pub fn entry(surface: impl Into<String>, cat: Cat) -> LexEntry {
    LexEntry::new(surface, cat)
}

impl LexEntry {
    pub fn new(surface: impl Into<String>, cat: Cat) -> Self {
        Self {
            surface: surface.into(),
            cat,
            animacy: None,
            agreement: None,
        }
    }

    pub fn surface(&self) -> &str {
        &self.surface
    }

    pub fn cat(&self) -> &Cat {
        &self.cat
    }

    pub fn animate(mut self) -> Self {
        self.animacy = Some(Animacy::Animate);
        self
    }

    pub fn inanimate(mut self) -> Self {
        self.animacy = Some(Animacy::Inanimate);
        self
    }

    pub fn with_agreement(mut self, person: Person, number: Number, gender: Gender) -> Self {
        self.agreement = Some(AgreementInfo {
            person,
            number,
            gender,
        });
        self
    }

    pub(crate) fn into_parts(self) -> (String, Cat, Option<Animacy>, Option<AgreementInfo>) {
        (self.surface, self.cat, self.animacy, self.agreement)
    }
}
