use crate::lexical::{
    Cardinality, ClauseKind, Modal, ObliqueSelection, PrepositionEntry, VerbEntry, VerbSelection,
};
use crate::syntax::{
    ArgumentStructure, Case, ClausalComplement, DeterminerPhrase, DeterminerPhraseKind, Finiteness,
    NegativePhrase, NonFiniteClause, ObliqueArgument, PerfectPhrase, PrepositionalComplement,
    PrepositionalPhrase, ProgressivePhrase, Tense, TensePhrase, VerbPhrase, VerbalProjection,
    Voice, VoicePhrase,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}

impl Diagnostic {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

impl From<Diagnostic> for DiagnosticBag {
    fn from(value: Diagnostic) -> Self {
        Self {
            diagnostics: vec![value],
        }
    }
}

impl fmt::Display for DiagnosticBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }
            write!(f, "[{}] {}", diagnostic.code, diagnostic.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for DiagnosticBag {}

pub type DerivationError = DiagnosticBag;

#[derive(Debug, Clone, PartialEq)]
pub struct FiniteDerivationSpec {
    pub tense: Tense,
    pub modal: Option<Modal>,
    pub negative: bool,
    pub perfect: bool,
    pub progressive: bool,
    pub passive: bool,
    pub reflexive: bool,
    pub causative_causer: Option<DeterminerPhrase>,
}

impl Default for FiniteDerivationSpec {
    fn default() -> Self {
        Self {
            tense: Tense::Present,
            modal: None,
            negative: false,
            perfect: false,
            progressive: false,
            passive: false,
            reflexive: false,
            causative_causer: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonFiniteDerivationSpec {
    pub finiteness: Finiteness,
    pub negative: bool,
    pub perfect: bool,
    pub progressive: bool,
    pub passive: bool,
}

impl Default for NonFiniteDerivationSpec {
    fn default() -> Self {
        Self {
            finiteness: Finiteness::ToInfinitive,
            negative: false,
            perfect: false,
            progressive: false,
            passive: false,
        }
    }
}

pub fn derive_tense_phrase(
    subject: DeterminerPhrase,
    predicate: VerbPhrase,
    spec: FiniteDerivationSpec,
) -> Result<TensePhrase, DiagnosticBag> {
    let mut diagnostics = validate_verb_phrase(
        &predicate,
        predicate.head.selection(),
        spec.passive,
        spec.reflexive,
    );

    if spec.causative_causer.is_some() && spec.passive {
        diagnostics.push(Diagnostic::new(
            "unsupported-combination",
            "causative and finite passive are not yet derived together",
        ));
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    if let Some(causer) = spec.causative_causer.clone() {
        return derive_causative_clause(causer, subject, predicate, spec);
    }

    let mut subject = subject;
    let mut predicate = predicate;

    if spec.reflexive {
        apply_reflexive(&subject, &mut predicate.arguments);
    }

    let voice = if spec.passive {
        apply_passive(&mut subject, &mut predicate.arguments);
        Voice::Passive
    } else {
        Voice::Active
    };

    let projection = build_projection(
        predicate,
        voice,
        spec.progressive,
        spec.perfect,
        spec.negative,
        spec.modal,
    );

    Ok(TensePhrase {
        subject,
        tense: spec.tense,
        predicate: projection,
    })
}

pub fn derive_non_finite_clause(
    predicate: VerbPhrase,
    spec: NonFiniteDerivationSpec,
) -> Result<NonFiniteClause, DiagnosticBag> {
    let diagnostics =
        validate_verb_phrase(&predicate, predicate.head.selection(), spec.passive, false);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let voice = if spec.passive {
        Voice::Passive
    } else {
        Voice::Active
    };

    Ok(NonFiniteClause {
        finiteness: spec.finiteness,
        predicate: build_projection(
            predicate,
            voice,
            spec.progressive,
            spec.perfect,
            spec.negative,
            None,
        ),
    })
}

fn derive_causative_clause(
    causer: DeterminerPhrase,
    causee: DeterminerPhrase,
    predicate: VerbPhrase,
    spec: FiniteDerivationSpec,
) -> Result<TensePhrase, DiagnosticBag> {
    let embedded = derive_non_finite_clause(
        predicate,
        NonFiniteDerivationSpec {
            finiteness: Finiteness::BareInfinitive,
            negative: false,
            perfect: spec.perfect,
            progressive: spec.progressive,
            passive: spec.passive,
        },
    )?;

    let matrix_predicate = VerbPhrase {
        head: VerbEntry::transitive("make").with_selection(
            VerbSelection::transitive().with_clause_kind(ClauseKind::BareInfinitive),
        ),
        particle: None,
        arguments: ArgumentStructure {
            direct_object: Some(Box::new(causee)),
            indirect_object: None,
            obliques: Vec::new(),
            adjuncts: Vec::new(),
            predicative_complement: None,
            clausal_complement: Some(ClausalComplement::NonFiniteClause(Box::new(embedded))),
        },
    };

    let projection = build_projection(
        matrix_predicate,
        Voice::Active,
        false,
        false,
        spec.negative,
        spec.modal,
    );

    Ok(TensePhrase {
        subject: causer,
        tense: spec.tense,
        predicate: projection,
    })
}

fn validate_verb_phrase(
    phrase: &VerbPhrase,
    selection: &VerbSelection,
    passive: bool,
    reflexive: bool,
) -> DiagnosticBag {
    let mut diagnostics = DiagnosticBag::default();

    match (
        selection.direct_object(),
        phrase.arguments.direct_object.is_some(),
    ) {
        (Cardinality::Required, false) if !passive => diagnostics.push(Diagnostic::new(
            "missing-direct-object",
            format!("verb `{}` requires a direct object", phrase.head.as_str()),
        )),
        (Cardinality::Forbidden, true) => diagnostics.push(Diagnostic::new(
            "forbidden-direct-object",
            format!(
                "verb `{}` does not license a direct object",
                phrase.head.as_str()
            ),
        )),
        _ => {}
    }

    match (
        selection.indirect_object(),
        phrase.arguments.indirect_object.is_some(),
    ) {
        (Cardinality::Required, false) => diagnostics.push(Diagnostic::new(
            "missing-indirect-object",
            format!(
                "verb `{}` requires an indirect object",
                phrase.head.as_str()
            ),
        )),
        (Cardinality::Forbidden, true) => diagnostics.push(Diagnostic::new(
            "forbidden-indirect-object",
            format!(
                "verb `{}` does not license an indirect object",
                phrase.head.as_str()
            ),
        )),
        _ => {}
    }

    if passive {
        if !selection.passive_allowed() {
            diagnostics.push(Diagnostic::new(
                "passive-not-licensed",
                format!(
                    "verb `{}` does not license passive voice",
                    phrase.head.as_str()
                ),
            ));
        }

        if phrase.arguments.direct_object.is_none() {
            diagnostics.push(Diagnostic::new(
                "passive-without-object",
                "passive derivation requires a direct object to promote",
            ));
        }
    }

    if reflexive
        && phrase.arguments.direct_object.is_none()
        && phrase.arguments.indirect_object.is_none()
    {
        diagnostics.push(Diagnostic::new(
            "reflexive-without-target",
            "reflexive derivation requires a DP object to rewrite",
        ));
    }

    if let Some(predicative) = &phrase.arguments.predicative_complement {
        if !selection
            .predicate_categories()
            .contains(&predicative.category())
        {
            diagnostics.push(Diagnostic::new(
                "predicate-not-licensed",
                format!(
                    "verb `{}` does not license this predicative complement",
                    phrase.head.as_str()
                ),
            ));
        }
    }

    if let Some(clausal) = &phrase.arguments.clausal_complement {
        let kind = clausal.clause_kind();
        if !selection.clausal_categories().contains(&kind) {
            diagnostics.push(Diagnostic::new(
                "clause-not-licensed",
                format!(
                    "verb `{}` does not license a {:?} clausal complement",
                    phrase.head.as_str(),
                    kind
                ),
            ));
        }
    }

    for oblique in &phrase.arguments.obliques {
        if !matches_oblique(selection.obliques(), oblique.phrase.head.as_str()) {
            diagnostics.push(Diagnostic::new(
                "oblique-not-licensed",
                format!(
                    "verb `{}` does not license oblique `{}`",
                    phrase.head.as_str(),
                    oblique.phrase.head.as_str()
                ),
            ));
        }
    }

    for required in selection
        .obliques()
        .iter()
        .filter(|selection| selection.required_flag())
    {
        let satisfied = phrase
            .arguments
            .obliques
            .iter()
            .any(|oblique| required.matches(oblique.phrase.head.as_str()));

        if !satisfied {
            diagnostics.push(Diagnostic::new(
                "missing-oblique",
                format!(
                    "verb `{}` requires oblique `{}`",
                    phrase.head.as_str(),
                    required.preposition().unwrap_or("<any>")
                ),
            ));
        }
    }

    diagnostics
}

fn matches_oblique(selections: &[ObliqueSelection], actual: &str) -> bool {
    selections.iter().any(|selection| selection.matches(actual))
}

fn apply_reflexive(subject: &DeterminerPhrase, arguments: &mut ArgumentStructure) {
    let antecedent = subject.semantics.reference.binding_key;
    let reflexive = DeterminerPhrase {
        kind: DeterminerPhraseKind::ReflexivePronoun { antecedent },
        semantics: subject
            .semantics
            .clone()
            .with_case(Case::Reflexive)
            .with_definiteness(crate::lexical::Definiteness::PronounLike)
            .with_countability(crate::lexical::Countability::PronounLike),
    };

    if arguments.direct_object.is_some() {
        arguments.direct_object = Some(Box::new(reflexive));
    } else if arguments.indirect_object.is_some() {
        arguments.indirect_object = Some(Box::new(reflexive));
    }
}

fn apply_passive(subject: &mut DeterminerPhrase, arguments: &mut ArgumentStructure) {
    if let Some(object) = arguments.direct_object.take() {
        let demoted_subject = std::mem::replace(subject, *object);
        arguments.obliques.push(
            ObliqueArgument::new(PrepositionalPhrase {
                head: PrepositionEntry::new("by"),
                complement: PrepositionalComplement::DeterminerPhrase(Box::new(demoted_subject)),
            })
            .with_role("agent"),
        );
    }
}

fn build_projection(
    predicate: VerbPhrase,
    voice: Voice,
    progressive: bool,
    perfect: bool,
    negative: bool,
    modal: Option<Modal>,
) -> VerbalProjection {
    let mut projection = VerbalProjection::Voice(VoicePhrase {
        head: voice,
        complement: Box::new(predicate),
    });

    if progressive {
        projection = VerbalProjection::Progressive(ProgressivePhrase {
            complement: Box::new(projection),
        });
    }

    if perfect {
        projection = VerbalProjection::Perfect(PerfectPhrase {
            complement: Box::new(projection),
        });
    }

    if negative {
        projection = VerbalProjection::Negative(NegativePhrase {
            complement: Box::new(projection),
        });
    }

    if let Some(head) = modal {
        projection = VerbalProjection::Modal(crate::syntax::ModalPhrase {
            head,
            complement: Box::new(projection),
        });
    }

    projection
}
