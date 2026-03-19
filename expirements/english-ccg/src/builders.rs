use std::ops::Add;

use english::{Animacy, Case, English, Gender, Number, Person};

use crate::cat::{bwd, fwd, Cat, VpForm};
use crate::derivation::{token, AgreementInfo, Ccg, TokenKind};
use crate::lexicon::LexEntry;

/// Coordinating conjunctions supported by the coordination helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conj {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrepRole {
    Complement,
    Adverbial,
    Adnominal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerbFormKind {
    Past,
    Present,
    Bare,
    Perfective,
    Progressive,
    PastParticipleAdj,
    PresentParticipleAdj,
    Passive,
    PassiveBy,
}

/// Whether an auxiliary inflects for finite agreement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxInflection {
    Invariant,
    Inflecting,
}

/// Builder for a lexical verb entry.
///
/// Finite forms preserve the underlying finite predicate category from the
/// lexical entry. Nonfinite forms rewrite the finite verbal result into an
/// explicit `VP[...]` category so auxiliaries can select them formally.
#[derive(Debug, Clone)]
pub struct VerbBuilder {
    lemma: String,
    cat: Cat,
}

/// Builder for prepositions whose category depends on syntactic role.
#[derive(Debug, Clone)]
pub struct PrepBuilder {
    lemma: String,
}

/// Builder for auxiliary lexical items.
///
/// The terminal method names are intentionally formal:
///
/// `<selected-complement-form>_<auxiliary-function>()`
///
/// For example, `past_participle_perfect()` denotes an inflecting auxiliary
/// that selects `VP[pastpart]` and contributes the perfect construction.
#[derive(Debug, Clone)]
pub struct AuxBuilder {
    surface: String,
    inflection: Option<AuxInflection>,
}

/// A feature bundle used to realize pronouns from lexical features rather than
/// from a user-chosen pronoun enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Referent {
    person: Person,
    number: Number,
    gender: Gender,
    animacy: Animacy,
}

impl Default for Referent {
    fn default() -> Self {
        Self {
            person: Person::Third,
            number: Number::Singular,
            gender: Gender::Neuter,
            animacy: Animacy::Inanimate,
        }
    }
}

impl Referent {
    /// Set grammatical person.
    pub fn person(mut self, person: Person) -> Self {
        self.person = person;
        self
    }

    /// Set grammatical number.
    pub fn number(mut self, number: Number) -> Self {
        self.number = number;
        self
    }

    /// Set grammatical gender.
    pub fn gender(mut self, gender: Gender) -> Self {
        self.gender = gender;
        self
    }

    /// Set animacy explicitly.
    pub fn animacy(mut self, animacy: Animacy) -> Self {
        self.animacy = animacy;
        self
    }

    /// Mark the referent as animate.
    pub fn animate(self) -> Self {
        self.animacy(Animacy::Animate)
    }

    /// Mark the referent as inanimate.
    pub fn inanimate(self) -> Self {
        self.animacy(Animacy::Inanimate)
    }

    pub(crate) fn agreement(self) -> AgreementInfo {
        AgreementInfo {
            person: self.person,
            number: self.number,
            gender: self.gender,
        }
    }

    pub(crate) fn person_value(self) -> Person {
        self.person
    }

    pub(crate) fn number_value(self) -> Number {
        self.number
    }

    pub(crate) fn gender_value(self) -> Gender {
        self.gender
    }

    pub(crate) fn animacy_value(self) -> Animacy {
        self.animacy
    }
}

impl Conj {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

impl VerbBuilder {
    /// Simple past finite form. Category remains the lexical finite category.
    pub fn past(self) -> Ccg {
        let cat = self.cat.clone();
        self.build(VerbFormKind::Past, cat)
    }

    /// Present finite form. Category remains the lexical finite category.
    pub fn present(self) -> Ccg {
        let cat = self.cat.clone();
        self.build(VerbFormKind::Present, cat)
    }

    /// Bare infinitival form.
    ///
    /// A finite predicate like `(S\NP)/NP` becomes `VP[bare]/NP`.
    pub fn bare(self) -> Ccg {
        let cat = replace_finite_predicate(&self.cat, Cat::VP(VpForm::Bare));
        self.build(VerbFormKind::Bare, cat)
    }

    /// Past participial verbal form.
    ///
    /// A finite predicate like `(S\NP)/NP` becomes `VP[pastpart]/NP`.
    pub fn perfective(self) -> Ccg {
        let cat = replace_finite_predicate(&self.cat, Cat::VP(VpForm::PastPart));
        self.build(VerbFormKind::Perfective, cat)
    }

    /// Present participial verbal form.
    ///
    /// A finite predicate like `(S\NP)/NP` becomes `VP[prespart]/NP`.
    pub fn progressive(self) -> Ccg {
        let cat = replace_finite_predicate(&self.cat, Cat::VP(VpForm::PresPart));
        self.build(VerbFormKind::Progressive, cat)
    }

    /// Adjectival past participle. Category is always `N/N`.
    pub fn past_participle(self) -> Ccg {
        self.build(VerbFormKind::PastParticipleAdj, fwd(Cat::N, Cat::N))
    }

    /// Adjectival present participle. Category is always `N/N`.
    pub fn present_participle(self) -> Ccg {
        self.build(VerbFormKind::PresentParticipleAdj, fwd(Cat::N, Cat::N))
    }

    /// Finite passive helper realized as a complete predicate `S\NP`.
    pub fn passive(self) -> Ccg {
        self.build(VerbFormKind::Passive, bwd(Cat::S, Cat::NP))
    }

    /// Finite passive helper with a rightward `by`-phrase slot.
    pub fn passive_by(self) -> Ccg {
        self.build(VerbFormKind::PassiveBy, fwd(bwd(Cat::S, Cat::NP), Cat::NP))
    }

    fn build(self, form: VerbFormKind, cat: Cat) -> Ccg {
        token(
            self.lemma.clone(),
            cat,
            TokenKind::Verb {
                lemma: self.lemma,
                form,
            },
            None,
            None,
        )
    }
}

impl AuxBuilder {
    /// Mark the auxiliary as morphologically invariant, as with `can` or
    /// `ought`.
    pub fn invariant(mut self) -> Self {
        self.inflection = Some(AuxInflection::Invariant);
        self
    }

    /// Mark the auxiliary as inflecting for finite agreement, as with `be`,
    /// `have`, or `do`.
    pub fn inflecting(mut self) -> Self {
        self.inflection = Some(AuxInflection::Inflecting);
        self
    }

    /// Build an auxiliary selecting `VP[bare]` and contributing modal force.
    ///
    /// Formal category: `(S\NP)/VP[bare]`
    pub fn bare_infinitive_modal(self) -> Ccg {
        self.build(fwd(bwd(Cat::S, Cat::NP), Cat::VP(VpForm::Bare)))
    }

    /// Build an auxiliary selecting `VP[to]` and contributing modal force.
    ///
    /// Formal category: `(S\NP)/VP[to]`
    pub fn to_infinitive_modal(self) -> Ccg {
        self.build(fwd(bwd(Cat::S, Cat::NP), Cat::VP(VpForm::To)))
    }

    /// Build an auxiliary selecting `VP[pastpart]` and contributing the
    /// perfect construction.
    ///
    /// Formal category: `(S\NP)/VP[pastpart]`
    pub fn past_participle_perfect(self) -> Ccg {
        self.build(fwd(bwd(Cat::S, Cat::NP), Cat::VP(VpForm::PastPart)))
    }

    /// Build an auxiliary selecting `VP[prespart]` and contributing the
    /// progressive construction.
    ///
    /// Formal category: `(S\NP)/VP[prespart]`
    pub fn present_participle_progressive(self) -> Ccg {
        self.build(fwd(bwd(Cat::S, Cat::NP), Cat::VP(VpForm::PresPart)))
    }

    /// Build an auxiliary selecting `VP[pastpart]` and contributing the
    /// passive construction.
    ///
    /// Formal category: `(S\NP)/VP[pastpart]`
    ///
    /// Note that `past_participle_passive()` and
    /// `past_participle_perfect()` currently share the same syntactic
    /// category. The distinction is formalized in the API naming and in
    /// realization intent, but not yet in the category inventory itself.
    pub fn past_participle_passive(self) -> Ccg {
        self.build(fwd(bwd(Cat::S, Cat::NP), Cat::VP(VpForm::PastPart)))
    }

    /// Build an auxiliary selecting `VP[bare]` and contributing support
    /// auxiliary behavior.
    ///
    /// Formal category: `(S\NP)/VP[bare]`
    pub fn bare_infinitive_support(self) -> Ccg {
        self.build(fwd(bwd(Cat::S, Cat::NP), Cat::VP(VpForm::Bare)))
    }

    fn build(self, cat: Cat) -> Ccg {
        token(
            self.surface.clone(),
            cat,
            TokenKind::Aux {
                inflection: self.inflection.unwrap_or_else(|| {
                    panic!(
                        "aux(\"{}\"): choose .invariant() or .inflecting() first",
                        self.surface
                    )
                }),
            },
            None,
            None,
        )
    }
}

impl PrepBuilder {
    pub fn complement(self) -> Ccg {
        token(
            self.lemma.clone(),
            fwd(Cat::PP, Cat::NP),
            TokenKind::Prep {
                _lemma: self.lemma,
                _role: PrepRole::Complement,
            },
            None,
            None,
        )
    }

    pub fn adverbial(self) -> Ccg {
        token(
            self.lemma.clone(),
            fwd(bwd(bwd(Cat::S, Cat::NP), bwd(Cat::S, Cat::NP)), Cat::NP),
            TokenKind::Prep {
                _lemma: self.lemma,
                _role: PrepRole::Adverbial,
            },
            None,
            None,
        )
    }

    pub fn adnominal(self) -> Ccg {
        token(
            self.lemma.clone(),
            fwd(bwd(Cat::N, Cat::N), Cat::NP),
            TokenKind::Prep {
                _lemma: self.lemma,
                _role: PrepRole::Adnominal,
            },
            None,
            None,
        )
    }
}

impl From<PrepBuilder> for Ccg {
    fn from(value: PrepBuilder) -> Self {
        value.complement()
    }
}

impl Add<Ccg> for PrepBuilder {
    type Output = Ccg;

    fn add(self, rhs: Ccg) -> Self::Output {
        self.complement() + rhs
    }
}

impl Add<PrepBuilder> for Ccg {
    type Output = Ccg;

    fn add(self, rhs: PrepBuilder) -> Self::Output {
        self + rhs.complement()
    }
}

/// Turn a proper-name lexical entry into a CCG item.
pub fn name(entry: &LexEntry) -> Ccg {
    token(
        entry.surface().to_string(),
        entry.cat().clone(),
        TokenKind::Name,
        entry.animacy().or(Some(Animacy::Animate)),
        entry.agreement().or(Some(AgreementInfo {
            person: Person::Third,
            number: Number::Singular,
            gender: Gender::Neuter,
        })),
    )
}

/// Start building a feature bundle for pronoun realization.
pub fn referent() -> Referent {
    Referent::default()
}

/// Realize a pronominal `NP` from referential features.
///
/// Case is determined later from the derivation, not chosen directly by the
/// user.
pub fn pro(referent: &Referent) -> Ccg {
    let agreement = referent.agreement();
    token(
        English::pronoun(
            &referent.person_value(),
            &referent.number_value(),
            &referent.gender_value(),
            &Case::Nominative,
        ),
        Cat::NP,
        TokenKind::Pronoun,
        Some(referent.animacy_value()),
        Some(agreement),
    )
}

/// Turn a common-noun lexical entry into a CCG item.
pub fn noun(entry: &LexEntry) -> Ccg {
    let surface = entry.surface().to_string();
    token(
        surface.clone(),
        entry.cat().clone(),
        TokenKind::Noun { lemma: surface },
        entry.animacy(),
        entry.agreement().or(Some(AgreementInfo {
            person: Person::Third,
            number: Number::Singular,
            gender: Gender::Neuter,
        })),
    )
}

/// Start building an auxiliary lexical item.
pub fn aux(surface: &str) -> AuxBuilder {
    AuxBuilder {
        surface: surface.to_string(),
        inflection: None,
    }
}

/// Determiner `NP/N`.
pub fn det(s: &str) -> Ccg {
    token(s, fwd(Cat::NP, Cat::N), TokenKind::Plain, None, None)
}

/// Relative pronoun `(N\N)/(S/NP)`.
pub fn rel(s: &str) -> Ccg {
    token(
        s,
        fwd(bwd(Cat::N, Cat::N), fwd(Cat::S, Cat::NP)),
        TokenKind::Plain,
        None,
        None,
    )
}

/// Turn a verb lexical entry into a verb-form builder.
pub fn verb(entry: &LexEntry) -> VerbBuilder {
    VerbBuilder {
        lemma: entry.surface().to_string(),
        cat: entry.cat().clone(),
    }
}

/// The infinitival marker `to : VP[to]/VP[bare]`.
///
/// This is a real lexical item in the derivation, not a stylistic realization
/// hint.
pub fn inf() -> Ccg {
    token(
        "to",
        fwd(Cat::VP(VpForm::To), Cat::VP(VpForm::Bare)),
        TokenKind::Plain,
        None,
        None,
    )
}

/// Prenominal adjective `N/N`.
pub fn adj(s: &str) -> Ccg {
    token(s, fwd(Cat::N, Cat::N), TokenKind::Plain, None, None)
}

/// VP modifier `(S\NP)\(S\NP)`.
pub fn adv(s: &str) -> Ccg {
    token(
        s,
        bwd(bwd(Cat::S, Cat::NP), bwd(Cat::S, Cat::NP)),
        TokenKind::Plain,
        None,
        None,
    )
}

/// Start building a preposition whose category depends on syntactic role.
pub fn prep(s: &str) -> PrepBuilder {
    PrepBuilder {
        lemma: s.to_string(),
    }
}

/// Complementizer `S/S`.
pub fn comp(s: &str) -> Ccg {
    token(s, fwd(Cat::S, Cat::S), TokenKind::Plain, None, None)
}

/// A phonologically empty gap item.
///
/// `gap(cat!(r"NP"))` is useful in relative-clause derivations, where it
/// contributes `NP/NP` internally so the gapped clause reduces to `S/NP`.
pub fn gap(cat: Cat) -> Ccg {
    token(
        "",
        fwd(cat.clone(), cat.clone()),
        TokenKind::Gap { original: cat },
        None,
        None,
    )
}

fn replace_finite_predicate(cat: &Cat, replacement: Cat) -> Cat {
    match cat {
        Cat::Bwd(result, arg) if **result == Cat::S && **arg == Cat::NP => replacement,
        Cat::Fwd(result, arg) => fwd(
            replace_finite_predicate(result, replacement),
            (**arg).clone(),
        ),
        _ => cat.clone(),
    }
}
