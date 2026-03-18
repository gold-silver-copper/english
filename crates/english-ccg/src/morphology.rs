use std::ops::Range;

use english::{Case, English, Form, Gender, Number, Person, Tense};

use crate::builders::{Modal, Pronoun, VerbFormKind};
use crate::derivation::{AgreementInfo, Token, TokenKind};

pub trait MorphLexicon {
    fn noun(&self, word: &str, number: Number) -> String;
    fn verb(
        &self,
        word: &str,
        person: Person,
        number: Number,
        tense: Tense,
        form: Form,
    ) -> String;
    fn pronoun(&self, person: Person, number: Number, gender: Gender, case: Case) -> String;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EnglishMorphology;

impl MorphLexicon for EnglishMorphology {
    fn noun(&self, word: &str, number: Number) -> String {
        English::noun(word, &number)
    }

    fn verb(
        &self,
        word: &str,
        person: Person,
        number: Number,
        tense: Tense,
        form: Form,
    ) -> String {
        English::verb(word, &person, &number, &tense, &form)
    }

    fn pronoun(&self, person: Person, number: Number, gender: Gender, case: Case) -> String {
        English::pronoun(&person, &number, &gender, &case).to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agreement {
    Auto,
    Override(Person, Number),
}

pub(crate) fn default_agreement() -> AgreementInfo {
    AgreementInfo {
        person: Person::Third,
        number: Number::Singular,
        gender: Gender::Neuter,
    }
}

pub(crate) fn override_agreement(agreement: Agreement) -> Option<AgreementInfo> {
    match agreement {
        Agreement::Auto => None,
        Agreement::Override(person, number) => Some(AgreementInfo {
            person,
            number,
            gender: Gender::Neuter,
        }),
    }
}

pub(crate) fn realize_token(
    token: &Token,
    agreement: AgreementInfo,
    subject_range: Option<&Range<usize>>,
    index: usize,
    morph: &impl MorphLexicon,
) -> Option<String> {
    let surface = match &token.kind {
        TokenKind::Plain | TokenKind::Name | TokenKind::Prep { .. } | TokenKind::Conj { .. } => {
            token.surface.clone()
        }
        TokenKind::Noun { lemma } => morph.noun(lemma, Number::Singular),
        TokenKind::Pronoun { pronoun } => realize_pronoun(*pronoun, subject_range, index, morph),
        TokenKind::Verb { lemma, form } => realize_verb(*form, lemma, agreement, morph),
        TokenKind::Modal { modal } => realize_modal(*modal, agreement, morph),
        TokenKind::Gap { .. } => return None,
    };
    if surface.is_empty() {
        None
    } else {
        Some(surface)
    }
}

fn realize_pronoun(
    pronoun: Pronoun,
    subject_range: Option<&Range<usize>>,
    index: usize,
    morph: &impl MorphLexicon,
) -> String {
    let agreement = pronoun.agreement();
    let case = if subject_range
        .map(|range| range.contains(&index))
        .unwrap_or(false)
    {
        Case::Nominative
    } else {
        Case::Accusative
    };
    morph.pronoun(
        agreement.person,
        agreement.number,
        agreement.gender,
        case,
    )
}

fn realize_verb(
    form: VerbFormKind,
    lemma: &str,
    agreement: AgreementInfo,
    morph: &impl MorphLexicon,
) -> String {
    match form {
        VerbFormKind::Past => morph.verb(
            lemma,
            Person::Third,
            Number::Singular,
            Tense::Past,
            Form::Finite,
        ),
        VerbFormKind::Present => morph.verb(
            lemma,
            agreement.person,
            agreement.number,
            Tense::Present,
            Form::Finite,
        ),
        VerbFormKind::Bare => morph.verb(
            lemma,
            Person::First,
            Number::Singular,
            Tense::Present,
            Form::Infinitive,
        ),
        VerbFormKind::Perfective | VerbFormKind::PastParticipleAdj => morph.verb(
            lemma,
            Person::First,
            Number::Singular,
            Tense::Past,
            Form::Participle,
        ),
        VerbFormKind::Progressive | VerbFormKind::PresentParticipleAdj => morph.verb(
            lemma,
            Person::First,
            Number::Singular,
            Tense::Present,
            Form::Participle,
        ),
        VerbFormKind::Passive => format!(
            "{} {}",
            morph.verb(
                "be",
                agreement.person,
                agreement.number,
                Tense::Past,
                Form::Finite
            ),
            morph.verb(
                lemma,
                Person::First,
                Number::Singular,
                Tense::Past,
                Form::Participle
            )
        ),
        VerbFormKind::PassiveBy => format!(
            "{} {} by",
            morph.verb(
                "be",
                agreement.person,
                agreement.number,
                Tense::Past,
                Form::Finite
            ),
            morph.verb(
                lemma,
                Person::First,
                Number::Singular,
                Tense::Past,
                Form::Participle
            )
        ),
    }
}

fn realize_modal(modal: Modal, agreement: AgreementInfo, morph: &impl MorphLexicon) -> String {
    match modal {
        Modal::Can => "can".to_string(),
        Modal::Must => "must".to_string(),
        Modal::Should => "should".to_string(),
        Modal::Will => "will".to_string(),
        Modal::Would => "would".to_string(),
        Modal::Have => morph.verb(
            "have",
            agreement.person,
            agreement.number,
            Tense::Present,
            Form::Finite,
        ),
        Modal::Be => morph.verb(
            "be",
            agreement.person,
            agreement.number,
            Tense::Present,
            Form::Finite,
        ),
    }
}
