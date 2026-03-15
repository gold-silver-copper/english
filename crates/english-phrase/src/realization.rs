use crate::derivation::{Diagnostic, DiagnosticBag};
use crate::lexical::{ComplementCategory, Modal};
use crate::syntax::*;
use english::{English, Form as MorphForm, Number, Person, Tense as MorphTense};

fn join_nonempty(parts: impl IntoIterator<Item = String>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn realize_all<T>(
    values: impl IntoIterator<Item = T>,
    mut f: impl FnMut(T) -> Result<String, DiagnosticBag>,
) -> Result<Vec<String>, DiagnosticBag> {
    let mut output = Vec::new();
    let mut bag = DiagnosticBag::default();

    for value in values {
        match f(value) {
            Ok(text) => output.push(text),
            Err(diagnostics) => {
                for diagnostic in diagnostics.iter() {
                    bag.push(diagnostic.clone());
                }
            }
        }
    }

    if bag.is_empty() { Ok(output) } else { Err(bag) }
}

pub fn realize_prepositional_phrase(phrase: &PrepositionalPhrase) -> Result<String, DiagnosticBag> {
    let category = match &phrase.complement {
        PrepositionalComplement::DeterminerPhrase(_) => ComplementCategory::DeterminerPhrase,
        PrepositionalComplement::ComplementizerPhrase(_) => {
            ComplementCategory::ComplementizerPhrase
        }
        PrepositionalComplement::NonFiniteClause(_) => ComplementCategory::NonFiniteClause,
    };

    if !phrase.head.complements().contains(&category) {
        return Err(Diagnostic::new(
            "preposition-selection",
            format!(
                "preposition `{}` does not license a {:?} complement",
                phrase.head.as_str(),
                category
            ),
        )
        .into());
    }

    let complement = match &phrase.complement {
        PrepositionalComplement::DeterminerPhrase(dp) => realize_determiner_phrase(dp)?,
        PrepositionalComplement::ComplementizerPhrase(cp) => realize_complementizer_phrase(cp)?,
        PrepositionalComplement::NonFiniteClause(clause) => realize_non_finite_clause(clause)?,
    };

    Ok(join_nonempty(vec![
        phrase.head.as_str().to_string(),
        complement,
    ]))
}

pub fn realize_adverb_phrase(phrase: &AdverbPhrase) -> Result<String, DiagnosticBag> {
    let mut parts = Vec::new();

    if let Some(specifier) = &phrase.specifier {
        parts.push(realize_adverb_phrase(specifier)?);
    }

    parts.push(phrase.head.as_str().to_string());
    parts.extend(realize_all(
        phrase.complements.iter(),
        realize_prepositional_phrase,
    )?);

    Ok(join_nonempty(parts))
}

pub fn realize_adjective_phrase(phrase: &AdjectivePhrase) -> Result<String, DiagnosticBag> {
    let mut parts = Vec::new();
    if let Some(specifier) = &phrase.specifier {
        parts.push(realize_adverb_phrase(specifier)?);
    }

    let degree = match phrase.degree {
        Degree::Positive => english::Degree::Positive,
        Degree::Comparative => english::Degree::Comparative,
        Degree::Superlative => english::Degree::Superlative,
    };
    parts.push(English::adj(phrase.head.lemma().as_str(), &degree));

    for complement in &phrase.complements {
        parts.push(match complement {
            AdjectiveComplement::PrepositionalPhrase(pp) => realize_prepositional_phrase(pp)?,
            AdjectiveComplement::ComplementizerPhrase(cp) => realize_complementizer_phrase(cp)?,
            AdjectiveComplement::NonFiniteClause(clause) => realize_non_finite_clause(clause)?,
        });
    }

    Ok(join_nonempty(parts))
}

pub fn realize_nominal_phrase(phrase: &NominalPhrase) -> Result<String, DiagnosticBag> {
    let mut parts = Vec::new();

    if let Quantity::Count(count) = phrase.quantity {
        parts.push(count.to_string());
    }

    parts.extend(realize_all(
        phrase.modifiers.iter(),
        realize_adjective_phrase,
    )?);

    let head = match &phrase.head {
        NominalHead::CommonNoun(noun) => match phrase.quantity {
            Quantity::Singular => English::noun(noun.lemma().as_str(), &Number::Singular),
            Quantity::Plural => English::noun(noun.lemma().as_str(), &Number::Plural),
            Quantity::Count(count) => {
                let number = if count == 1 {
                    Number::Singular
                } else {
                    Number::Plural
                };
                English::noun(noun.lemma().as_str(), &number)
            }
        },
        NominalHead::ProperName(name) => name.clone(),
    };
    parts.push(head);

    for complement in &phrase.complements {
        parts.push(match complement {
            NominalComplement::PrepositionalPhrase(pp) => realize_prepositional_phrase(pp)?,
        });
    }

    for postmodifier in &phrase.postmodifiers {
        parts.push(match postmodifier {
            NominalPostmodifier::PrepositionalPhrase(pp) => realize_prepositional_phrase(pp)?,
            NominalPostmodifier::ComplementizerPhrase(cp) => realize_complementizer_phrase(cp)?,
            NominalPostmodifier::NonFiniteClause(clause) => realize_non_finite_clause(clause)?,
            NominalPostmodifier::RelativeClause(clause) => realize_relative_clause(clause)?,
        });
    }

    Ok(join_nonempty(parts))
}

fn realize_possessor(specifier: &DeterminerPhrase) -> Result<String, DiagnosticBag> {
    match &specifier.kind {
        DeterminerPhraseKind::BarePronoun { pronoun } => Ok(pronoun.possessive_determiner().into()),
        _ => {
            let text = realize_determiner_phrase(specifier)?;
            Ok(if text.is_empty() {
                String::new()
            } else {
                English::add_possessive(&text)
            })
        }
    }
}

pub fn realize_determiner_phrase(phrase: &DeterminerPhrase) -> Result<String, DiagnosticBag> {
    match &phrase.kind {
        DeterminerPhraseKind::Projection(projection) => {
            let mut parts = Vec::new();

            if let Some(specifier) = &projection.specifier {
                parts.push(realize_possessor(specifier)?);
            }

            match &projection.head {
                DeterminerHead::Overt(entry) => {
                    parts.push(entry.head().render().to_string());
                }
                DeterminerHead::Silent(_) => {}
            }

            parts.push(realize_nominal_phrase(&projection.nominal)?);
            Ok(join_nonempty(parts))
        }
        DeterminerPhraseKind::BarePronoun { pronoun } => Ok(pronoun.render().to_string()),
        DeterminerPhraseKind::ReflexivePronoun { .. } => {
            Ok(phrase.semantics.reflexive_form().to_string())
        }
        DeterminerPhraseKind::Gap { .. } => Ok(String::new()),
    }
}

#[derive(Debug, Clone)]
struct ProjectionSpec<'a> {
    modal: Option<Modal>,
    negative: bool,
    perfect: bool,
    progressive: bool,
    voice: Voice,
    verb: &'a VerbPhrase,
}

#[derive(Debug, Clone)]
struct VerbKernel {
    head: String,
    tail: Vec<String>,
    is_auxiliary: bool,
}

fn collect_projection<'a>(
    projection: &'a VerbalProjection,
    spec: &mut ProjectionSpec<'a>,
    diagnostics: &mut DiagnosticBag,
) {
    match projection {
        VerbalProjection::Modal(phrase) => {
            if spec.modal.replace(phrase.head).is_some() {
                diagnostics.push(Diagnostic::new(
                    "duplicate-modal",
                    "verbal projection contains more than one modal head",
                ));
            }
            collect_projection(&phrase.complement, spec, diagnostics);
        }
        VerbalProjection::Negative(phrase) => {
            if spec.negative {
                diagnostics.push(Diagnostic::new(
                    "duplicate-negative",
                    "verbal projection contains more than one negative head",
                ));
            }
            spec.negative = true;
            collect_projection(&phrase.complement, spec, diagnostics);
        }
        VerbalProjection::Perfect(phrase) => {
            if spec.perfect {
                diagnostics.push(Diagnostic::new(
                    "duplicate-perfect",
                    "verbal projection contains more than one perfect head",
                ));
            }
            spec.perfect = true;
            collect_projection(&phrase.complement, spec, diagnostics);
        }
        VerbalProjection::Progressive(phrase) => {
            if spec.progressive {
                diagnostics.push(Diagnostic::new(
                    "duplicate-progressive",
                    "verbal projection contains more than one progressive head",
                ));
            }
            spec.progressive = true;
            collect_projection(&phrase.complement, spec, diagnostics);
        }
        VerbalProjection::Voice(phrase) => {
            spec.voice = phrase.head;
            spec.verb = &phrase.complement;
        }
        VerbalProjection::Verb(verb) => {
            spec.voice = Voice::Active;
            spec.verb = verb;
        }
    }
}

fn morph_tense(tense: Tense) -> MorphTense {
    match tense {
        Tense::Present => MorphTense::Present,
        Tense::Past => MorphTense::Past,
    }
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

fn base_form(lemma: &str) -> String {
    English::verb(
        lemma,
        &Person::Third,
        &Number::Singular,
        &MorphTense::Present,
        &MorphForm::Infinitive,
    )
}

fn present_participle(lemma: &str) -> String {
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

fn lower_kernel(kernel: &VerbKernel, participle_is_past: bool) -> Vec<String> {
    let head = if participle_is_past {
        past_participle(&kernel.head)
    } else {
        present_participle(&kernel.head)
    };

    let mut parts = vec![head];
    parts.extend(kernel.tail.clone());
    parts
}

fn wrap_auxiliary(aux_lemma: &str, complement: Vec<String>) -> VerbKernel {
    VerbKernel {
        head: aux_lemma.to_string(),
        tail: complement,
        is_auxiliary: true,
    }
}

fn build_kernel(spec: &ProjectionSpec<'_>) -> VerbKernel {
    let mut kernel = VerbKernel {
        head: spec.verb.head.as_str().to_string(),
        tail: spec
            .verb
            .particle
            .as_ref()
            .map(|particle| vec![particle.as_str().to_string()])
            .unwrap_or_default(),
        is_auxiliary: false,
    };

    if spec.voice == Voice::Passive {
        kernel = wrap_auxiliary("be", lower_kernel(&kernel, true));
    }

    if spec.progressive {
        kernel = wrap_auxiliary("be", lower_kernel(&kernel, false));
    }

    if spec.perfect {
        kernel = wrap_auxiliary("have", lower_kernel(&kernel, true));
    }

    kernel
}

fn realize_verb_complex(
    spec: &ProjectionSpec<'_>,
    tense: Option<Tense>,
    finiteness: Finiteness,
    subject: Option<&DeterminerPhrase>,
) -> Result<Vec<String>, DiagnosticBag> {
    let kernel = build_kernel(spec);

    let agreement = subject
        .map(|subject| subject.semantics.agreement_tuple())
        .unwrap_or((Person::Third, Number::Singular));

    Ok(match finiteness {
        Finiteness::Finite => {
            let tense = tense.ok_or_else(|| {
                DiagnosticBag::from(Diagnostic::new(
                    "missing-tense",
                    "finite realization requires a tense value",
                ))
            })?;

            if let Some(modal) = spec.modal {
                let mut parts = vec![modal.render().to_string()];
                if spec.negative {
                    parts.push("not".to_string());
                }
                parts.push(base_form(&kernel.head));
                parts.extend(kernel.tail);
                parts
            } else {
                let use_do_support = spec.negative && !kernel.is_auxiliary && kernel.head != "be";
                if use_do_support {
                    let mut parts = vec![finite_form("do", &agreement.0, &agreement.1, tense)];
                    parts.push("not".to_string());
                    parts.push(base_form(&kernel.head));
                    parts.extend(kernel.tail);
                    parts
                } else {
                    let mut parts =
                        vec![finite_form(&kernel.head, &agreement.0, &agreement.1, tense)];
                    if spec.negative {
                        parts.push("not".to_string());
                    }
                    parts.extend(kernel.tail);
                    parts
                }
            }
        }
        Finiteness::BareInfinitive => {
            let mut parts = Vec::new();
            if spec.negative {
                parts.push("not".to_string());
            }
            parts.push(base_form(&kernel.head));
            parts.extend(kernel.tail);
            parts
        }
        Finiteness::ToInfinitive => {
            let mut parts = Vec::new();
            if spec.negative {
                parts.push("not".to_string());
            }
            parts.push("to".to_string());
            parts.push(base_form(&kernel.head));
            parts.extend(kernel.tail);
            parts
        }
        Finiteness::GerundParticiple => {
            let mut parts = Vec::new();
            if spec.negative {
                parts.push("not".to_string());
            }
            parts.push(present_participle(&kernel.head));
            parts.extend(kernel.tail);
            parts
        }
    })
}

fn realize_predicate_complement(complement: &PredicateComplement) -> Result<String, DiagnosticBag> {
    match complement {
        PredicateComplement::AdjectivePhrase(ap) => realize_adjective_phrase(ap),
        PredicateComplement::DeterminerPhrase(dp) => realize_determiner_phrase(dp),
    }
}

fn realize_clausal_complement(complement: &ClausalComplement) -> Result<String, DiagnosticBag> {
    match complement {
        ClausalComplement::ComplementizerPhrase(cp) => realize_complementizer_phrase(cp),
        ClausalComplement::NonFiniteClause(clause) => realize_non_finite_clause(clause),
    }
}

fn realize_verb_adjunct(adjunct: &VerbAdjunct) -> Result<String, DiagnosticBag> {
    match adjunct {
        VerbAdjunct::PrepositionalPhrase(pp) => realize_prepositional_phrase(pp),
        VerbAdjunct::AdverbPhrase(advp) => realize_adverb_phrase(advp),
    }
}

fn realize_projection_with_context(
    projection: &VerbalProjection,
    tense: Option<Tense>,
    finiteness: Finiteness,
    subject: Option<&DeterminerPhrase>,
) -> Result<String, DiagnosticBag> {
    let mut diagnostics = DiagnosticBag::default();
    let default_verb = VerbPhrase {
        head: crate::lexical::VerbEntry::new("do"),
        particle: None,
        arguments: ArgumentStructure::default(),
    };
    let mut spec = ProjectionSpec {
        modal: None,
        negative: false,
        perfect: false,
        progressive: false,
        voice: Voice::Active,
        verb: &default_verb,
    };
    collect_projection(projection, &mut spec, &mut diagnostics);

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut parts = realize_verb_complex(&spec, tense, finiteness, subject)?;

    if let Some(indirect_object) = &spec.verb.arguments.indirect_object {
        parts.push(realize_determiner_phrase(indirect_object)?);
    }

    if let Some(direct_object) = &spec.verb.arguments.direct_object {
        parts.push(realize_determiner_phrase(direct_object)?);
    }

    if let Some(predicative) = &spec.verb.arguments.predicative_complement {
        parts.push(realize_predicate_complement(predicative)?);
    }

    if let Some(clausal) = &spec.verb.arguments.clausal_complement {
        parts.push(realize_clausal_complement(clausal)?);
    }

    for oblique in &spec.verb.arguments.obliques {
        parts.push(realize_prepositional_phrase(&oblique.phrase)?);
    }

    for adjunct in &spec.verb.arguments.adjuncts {
        parts.push(realize_verb_adjunct(adjunct)?);
    }

    Ok(join_nonempty(parts))
}

pub fn realize_verb_phrase(phrase: &VerbPhrase) -> Result<String, DiagnosticBag> {
    realize_projection_with_context(
        &VerbalProjection::Verb(phrase.clone()),
        None,
        Finiteness::BareInfinitive,
        None,
    )
}

pub fn realize_tense_phrase(phrase: &TensePhrase) -> Result<String, DiagnosticBag> {
    Ok(join_nonempty(vec![
        realize_determiner_phrase(&phrase.subject)?,
        realize_projection_with_context(
            &phrase.predicate,
            Some(phrase.tense),
            Finiteness::Finite,
            Some(&phrase.subject),
        )?,
    ]))
}

pub fn realize_non_finite_clause(clause: &NonFiniteClause) -> Result<String, DiagnosticBag> {
    realize_projection_with_context(&clause.predicate, None, clause.finiteness, None)
}

pub fn realize_complementizer_phrase(
    phrase: &ComplementizerPhrase,
) -> Result<String, DiagnosticBag> {
    Ok(join_nonempty(vec![
        phrase.head.render().to_string(),
        realize_tense_phrase(&phrase.complement)?,
    ]))
}

pub fn realize_relative_clause(clause: &RelativeClause) -> Result<String, DiagnosticBag> {
    Ok(join_nonempty(vec![
        clause.marker.render().to_string(),
        realize_tense_phrase(&clause.clause)?,
    ]))
}

pub fn realize_clause(clause: &Clause) -> Result<String, DiagnosticBag> {
    realize_tense_phrase(&clause.tense_phrase)
}

pub fn realize_sentence(sentence: &Sentence) -> Result<String, DiagnosticBag> {
    let mut text = realize_clause(&sentence.clause)?;
    if sentence.capitalize {
        text = English::capitalize_first(&text);
    }
    if let Some(terminal) = sentence.terminal {
        let mark = match terminal {
            Terminal::Period => ".",
            Terminal::QuestionMark => "?",
            Terminal::ExclamationMark => "!",
        };
        text.push_str(mark);
    }
    Ok(text)
}
