use english::{Animacy, Gender, Number, Person};

use crate::{cat, entry, Cat, LexEntry};

/// Convenience constructor for a singular proper-name `NP`.
pub fn proper(surface: &str) -> LexEntry {
    entry(surface, Cat::NP).animate().with_agreement(
        Person::Third,
        Number::Singular,
        Gender::Neuter,
    )
}

/// Convenience constructor for a singular common noun `N`.
pub fn common(surface: &str, animacy: Animacy) -> LexEntry {
    let base =
        entry(surface, Cat::N).with_agreement(Person::Third, Number::Singular, Gender::Neuter);
    match animacy {
        Animacy::Animate => base.animate(),
        Animacy::Inanimate => base.inanimate(),
    }
}

/// Convenience constructor for an intransitive finite verb `S\NP`.
pub fn iv(surface: &str) -> LexEntry {
    entry(surface, cat!(r"S\NP"))
}

/// Convenience constructor for a transitive finite verb `(S\NP)/NP`.
pub fn tv(surface: &str) -> LexEntry {
    entry(surface, cat!(r"(S\NP)/NP"))
}

/// Convenience constructor for a sentential complement verb `(S\NP)/S`.
pub fn scomp(surface: &str) -> LexEntry {
    entry(surface, cat!(r"(S\NP)/S"))
}

/// Convenience constructor for a verb selecting `VP[to]`, such as "promise".
pub fn vpcomp(surface: &str) -> LexEntry {
    entry(surface, cat!(r"(S\NP)/VP[to]"))
}
