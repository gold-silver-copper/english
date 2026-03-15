use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, DeterminerHead, DeterminerPhrase, Phrase, PrepositionalPhrase,
    Tense, VerbForm, VerbPhrase,
};
use english::{English, Form as MorphForm, Number, Person, Tense as MorphTense};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizationError {
    message: String,
}

impl RealizationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RealizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for RealizationError {}

pub type RealizationResult<T> = Result<T, RealizationError>;

fn join_nonempty(parts: impl IntoIterator<Item = String>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn morph_tense(tense: Tense) -> MorphTense {
    match tense {
        Tense::Present => MorphTense::Present,
        Tense::Past => MorphTense::Past,
    }
}

fn agreement(subject: Option<&DeterminerPhrase>) -> (Person, Number) {
    match subject {
        Some(phrase) => match phrase.head {
            DeterminerHead::Pronoun(pronoun) => (pronoun.person(), pronoun.number()),
            _ => (Person::Third, phrase.number.clone()),
        },
        None => (Person::Third, Number::Singular),
    }
}

fn base_form(lemma: &str) -> String {
    English::verb(
        lemma,
        &Person::Third,
        &Number::Singular,
        &MorphTense::Present,
        &MorphForm::Infinitive,
    )
}

fn finite_form(lemma: &str, person: &Person, number: &Number, tense: Tense) -> String {
    English::verb(
        lemma,
        person,
        number,
        &morph_tense(tense),
        &MorphForm::Finite,
    )
}

fn gerund_form(lemma: &str) -> String {
    English::verb(
        lemma,
        &Person::Third,
        &Number::Singular,
        &MorphTense::Present,
        &MorphForm::Participle,
    )
}

fn past_participle(lemma: &str) -> String {
    English::verb(
        lemma,
        &Person::Third,
        &Number::Singular,
        &MorphTense::Past,
        &MorphForm::Participle,
    )
}

fn realize_modifier(phrase: &Phrase) -> RealizationResult<String> {
    realize_phrase(phrase)
}

fn realize_suffixes(phrases: &[Box<Phrase>]) -> RealizationResult<Vec<String>> {
    phrases
        .iter()
        .map(|phrase| realize_phrase(phrase.as_ref()))
        .collect()
}

fn realize_verb_words(
    phrase: &VerbPhrase,
    subject: Option<&DeterminerPhrase>,
) -> RealizationResult<Vec<String>> {
    let lemma = phrase.head.as_str();
    let (person, number) = agreement(subject);

    Ok(match phrase.form {
        VerbForm::Finite(tense) => {
            if phrase.negative && lemma != "be" {
                vec![
                    finite_form("do", &person, &number, tense),
                    "not".to_string(),
                    base_form(lemma),
                ]
            } else {
                let mut words = vec![finite_form(lemma, &person, &number, tense)];
                if phrase.negative {
                    words.push("not".to_string());
                }
                words
            }
        }
        VerbForm::BareInfinitive => {
            let mut words = Vec::new();
            if phrase.negative {
                words.push("not".to_string());
            }
            words.push(base_form(lemma));
            words
        }
        VerbForm::ToInfinitive => {
            let mut words = Vec::new();
            if phrase.negative {
                words.push("not".to_string());
            }
            words.push("to".to_string());
            words.push(base_form(lemma));
            words
        }
        VerbForm::GerundParticiple => {
            let mut words = Vec::new();
            if phrase.negative {
                words.push("not".to_string());
            }
            words.push(gerund_form(lemma));
            words
        }
        VerbForm::PastParticiple => {
            let mut words = Vec::new();
            if phrase.negative {
                words.push("not".to_string());
            }
            words.push(past_participle(lemma));
            words
        }
    })
}

pub fn realize_phrase(phrase: &Phrase) -> RealizationResult<String> {
    match phrase {
        Phrase::DP(dp) => realize_determiner_phrase(dp),
        Phrase::VP(vp) => realize_verb_phrase(vp),
        Phrase::PP(pp) => realize_prepositional_phrase(pp),
        Phrase::AdjP(adjp) => realize_adjective_phrase(adjp),
        Phrase::AdvP(advp) => realize_adverb_phrase(advp),
    }
}

pub fn realize_determiner_phrase(phrase: &DeterminerPhrase) -> RealizationResult<String> {
    let mut parts = Vec::new();

    let modifiers = phrase
        .modifiers
        .iter()
        .map(|modifier| realize_modifier(modifier.as_ref()))
        .collect::<Result<Vec<_>, _>>()?;

    let head = match &phrase.head {
        DeterminerHead::CommonNoun(noun) => {
            if let Some(determiner) = phrase.determiner {
                parts.push(determiner.as_str().to_string());
            }
            parts.extend(modifiers);
            English::noun(noun.as_str(), &phrase.number)
        }
        DeterminerHead::ProperName(name) => {
            if phrase.determiner.is_some() {
                return Err(RealizationError::new(
                    "proper names do not take determiners in the simplified DP model",
                ));
            }
            parts.extend(modifiers);
            name.clone()
        }
        DeterminerHead::Pronoun(pronoun) => {
            if phrase.determiner.is_some() {
                return Err(RealizationError::new(
                    "pronouns do not take determiners in the simplified DP model",
                ));
            }
            parts.extend(modifiers);
            pronoun.as_str().to_string()
        }
    };

    parts.push(head);
    parts.extend(realize_suffixes(&phrase.complements)?);
    Ok(join_nonempty(parts))
}

pub fn realize_adjective_phrase(phrase: &AdjectivePhrase) -> RealizationResult<String> {
    let mut parts = Vec::new();

    if let Some(modifier) = &phrase.modifier {
        match modifier.as_ref() {
            Phrase::AdvP(_) => parts.push(realize_phrase(modifier)?),
            _ => {
                return Err(RealizationError::new(
                    "adjective phrase modifiers must be adverb phrases",
                ));
            }
        }
    }

    parts.push(English::adj(
        phrase.head.as_str(),
        &english::Degree::Positive,
    ));
    parts.extend(realize_suffixes(&phrase.complements)?);
    Ok(join_nonempty(parts))
}

pub fn realize_adverb_phrase(phrase: &AdverbPhrase) -> RealizationResult<String> {
    let mut parts = Vec::new();

    if let Some(modifier) = &phrase.modifier {
        match modifier.as_ref() {
            Phrase::AdvP(_) => parts.push(realize_phrase(modifier)?),
            _ => {
                return Err(RealizationError::new(
                    "adverb phrase modifiers must be adverb phrases",
                ));
            }
        }
    }

    parts.push(phrase.head.as_str().to_string());
    parts.extend(realize_suffixes(&phrase.complements)?);
    Ok(join_nonempty(parts))
}

pub fn realize_prepositional_phrase(phrase: &PrepositionalPhrase) -> RealizationResult<String> {
    Ok(join_nonempty(vec![
        phrase.head.as_str().to_string(),
        realize_phrase(phrase.complement.as_ref())?,
    ]))
}

pub fn realize_verb_phrase(phrase: &VerbPhrase) -> RealizationResult<String> {
    let mut parts = realize_verb_words(phrase, None)?;
    parts.extend(realize_suffixes(&phrase.complements)?);
    parts.extend(realize_suffixes(&phrase.adjuncts)?);
    Ok(join_nonempty(parts))
}

pub fn realize_clause(
    subject: &DeterminerPhrase,
    predicate: &VerbPhrase,
) -> RealizationResult<String> {
    if !matches!(predicate.form, VerbForm::Finite(_)) {
        return Err(RealizationError::new(
            "finite clauses require a finite verb phrase",
        ));
    }

    let mut parts = Vec::new();
    parts.push(realize_determiner_phrase(subject)?);
    parts.extend(realize_verb_words(predicate, Some(subject))?);
    parts.extend(realize_suffixes(&predicate.complements)?);
    parts.extend(realize_suffixes(&predicate.adjuncts)?);
    Ok(join_nonempty(parts))
}

pub fn realize_sentence(
    subject: &DeterminerPhrase,
    predicate: &VerbPhrase,
) -> RealizationResult<String> {
    let mut text = realize_clause(subject, predicate)?;
    text = English::capitalize_first(&text);
    text.push('.');
    Ok(text)
}
