use english::{Animacy, Gender, Number, Person};

use crate::{cat, entry, Cat, LexEntry};

pub fn proper(surface: &str) -> LexEntry {
    entry(surface, Cat::NP).animate().with_agreement(
        Person::Third,
        Number::Singular,
        Gender::Neuter,
    )
}

pub fn common(surface: &str, animacy: Animacy) -> LexEntry {
    let base =
        entry(surface, Cat::N).with_agreement(Person::Third, Number::Singular, Gender::Neuter);
    match animacy {
        Animacy::Animate => base.animate(),
        Animacy::Inanimate => base.inanimate(),
    }
}

pub fn iv(surface: &str) -> LexEntry {
    entry(surface, cat!("S\\NP"))
}

pub fn tv(surface: &str) -> LexEntry {
    entry(surface, cat!("(S\\NP)/NP"))
}

pub fn scomp(surface: &str) -> LexEntry {
    entry(surface, cat!("(S\\NP)/S"))
}

pub fn vpcomp(surface: &str) -> LexEntry {
    entry(surface, cat!("(S\\NP)/(S\\NP)"))
}
