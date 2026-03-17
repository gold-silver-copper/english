use crate::lexical::{Determiner, Pronoun};
use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, ClauseForm, Complementizer, ComplementizerPhrase,
    DeterminerPhrase, NominalDeterminerPhrase, NounPhrase, Phrase, PrepositionalPhrase,
    PronominalDeterminerPhrase, Tense, TensePhrase, VerbForm, VerbPhrase,
};
use english::{English, Form as MorphForm, Number, Person, Tense as MorphTense};

mod private {
    pub trait Sealed {}
}

pub trait Realizable: private::Sealed {
    fn realize(&self) -> String {
        self.realize_with(RealizationOptions::default())
    }

    fn realize_with(&self, options: RealizationOptions) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminal {
    Period,
    QuestionMark,
    ExclamationMark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RealizationOptions {
    capitalize: bool,
    terminal: Option<Terminal>,
}

impl RealizationOptions {
    pub fn sentence() -> Self {
        Self {
            capitalize: true,
            terminal: Some(Terminal::Period),
        }
    }

    pub fn capitalize(mut self) -> Self {
        self.capitalize = true;
        self
    }

    pub fn lowercase(mut self) -> Self {
        self.capitalize = false;
        self
    }

    pub fn terminal(mut self, terminal: Terminal) -> Self {
        self.terminal = Some(terminal);
        self
    }

    pub fn without_terminal(mut self) -> Self {
        self.terminal = None;
        self
    }

    pub fn period(self) -> Self {
        self.terminal(Terminal::Period)
    }

    pub fn question_mark(self) -> Self {
        self.terminal(Terminal::QuestionMark)
    }

    pub fn exclamation_mark(self) -> Self {
        self.terminal(Terminal::ExclamationMark)
    }

    pub fn capitalize_flag(&self) -> bool {
        self.capitalize
    }

    pub fn terminal_opt(&self) -> Option<Terminal> {
        self.terminal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DpRenderRole {
    Subject,
    Object,
    PossessiveDependent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SubjectPosition {
    None,
    Trace,
}

fn join_nonempty(parts: impl IntoIterator<Item = String>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn apply_realization_options(mut text: String, options: RealizationOptions) -> String {
    if options.capitalize_flag() {
        text = English::capitalize_first(&text);
    }

    if let Some(terminal) = options.terminal_opt() {
        text.push(match terminal {
            Terminal::Period => '.',
            Terminal::QuestionMark => '?',
            Terminal::ExclamationMark => '!',
        });
    }

    text
}

fn indefinite_article(next: &str) -> &'static str {
    match next
        .chars()
        .find(|ch| ch.is_ascii_alphabetic())
        .map(|ch| ch.to_ascii_lowercase())
    {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

fn morph_tense(tense: Tense) -> MorphTense {
    match tense {
        Tense::Present => MorphTense::Present,
        Tense::Past => MorphTense::Past,
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

fn render_phrase_list(values: &[Box<Phrase>]) -> Vec<String> {
    values
        .iter()
        .map(|value| render_phrase_in_context(value.as_ref(), DpRenderRole::Object))
        .collect()
}

fn render_complement(phrase: &Phrase) -> String {
    render_phrase_in_context(phrase, DpRenderRole::Object)
}

fn render_complement_list(values: &[Box<Phrase>]) -> Vec<String> {
    values
        .iter()
        .map(|value| render_complement(value.as_ref()))
        .collect()
}

fn agreement_from_dp(dp: &DeterminerPhrase) -> (Person, Number) {
    match dp {
        DeterminerPhrase::BareNominal(nominal)
        | DeterminerPhrase::DeterminedNominal { nominal, .. }
        | DeterminerPhrase::PossessedNominal { nominal, .. } => {
            (Person::Third, nominal.number().clone())
        }
        DeterminerPhrase::ProperName(_) => (Person::Third, Number::Singular),
        DeterminerPhrase::Pronoun { pronoun, .. } => (pronoun.person(), pronoun.number()),
    }
}

fn render_phrase(phrase: &Phrase) -> String {
    render_phrase_in_context(phrase, DpRenderRole::Subject)
}

fn render_phrase_in_context(phrase: &Phrase, dp_role: DpRenderRole) -> String {
    match phrase {
        Phrase::CP(cp) => render_cp(cp),
        Phrase::TP(tp) => render_tp(tp),
        Phrase::DP(dp) => render_dp(dp, dp_role),
        Phrase::NP(np) => render_np(np),
        Phrase::VP(vp) => render_vp(vp, SubjectPosition::None),
        Phrase::PP(pp) => render_pp(pp),
        Phrase::AdjP(ap) => render_ap(ap),
        Phrase::AdvP(advp) => render_advp(advp),
    }
}

fn render_cp(cp: &ComplementizerPhrase) -> String {
    let mut parts = Vec::new();

    if let Some(specifier) = cp.specifier_opt() {
        parts.push(render_phrase(specifier));
    }

    if let Some(head) = render_complementizer(cp.head()) {
        parts.push(head.to_string());
    }

    parts.push(render_tp(cp.complement()));
    join_nonempty(parts)
}

fn render_complementizer(head: Complementizer) -> Option<&'static str> {
    match head {
        Complementizer::Null => None,
        Complementizer::That => Some("that"),
        Complementizer::Whether => Some("whether"),
        Complementizer::If => Some("if"),
    }
}

fn render_tp<Form: ClauseForm>(tp: &TensePhrase<Form>) -> String {
    let subject = tp.subject_opt();
    let surfaced_subject = subject.map(|dp| render_dp(dp, DpRenderRole::Subject));

    let predicate = render_tense_head(
        tp.predicate(),
        tp.form(),
        tp.is_negative(),
        subject,
        subject.map_or(SubjectPosition::None, |_| SubjectPosition::Trace),
    );

    join_nonempty(
        surfaced_subject
            .into_iter()
            .chain(std::iter::once(predicate)),
    )
}

fn render_tense_head(
    predicate: &VerbPhrase,
    form: VerbForm,
    negative: bool,
    subject: Option<&DeterminerPhrase>,
    subject_position: SubjectPosition,
) -> String {
    let lemma = predicate.head().as_str();
    let neg_count = usize::from(negative);

    let mut parts = match form {
        VerbForm::Finite(tense) => {
            let agreement = subject
                .map(agreement_from_dp)
                .unwrap_or((Person::Third, Number::Singular));

            if neg_count > 0 && lemma != "be" {
                let mut words = vec![finite_form("do", &agreement.0, &agreement.1, tense)];
                words.extend(std::iter::repeat_n("not".to_string(), neg_count));
                words.push(base_form(lemma));
                words
            } else {
                let mut words = vec![finite_form(lemma, &agreement.0, &agreement.1, tense)];
                words.extend(std::iter::repeat_n("not".to_string(), neg_count));
                words
            }
        }
        VerbForm::BareInfinitive => {
            let mut words = Vec::new();
            words.extend(std::iter::repeat_n("not".to_string(), neg_count));
            words.push(base_form(lemma));
            words
        }
        VerbForm::ToInfinitive => {
            let mut words = Vec::new();
            words.extend(std::iter::repeat_n("not".to_string(), neg_count));
            words.push("to".to_string());
            words.push(base_form(lemma));
            words
        }
        VerbForm::GerundParticiple => {
            let mut words = Vec::new();
            words.extend(std::iter::repeat_n("not".to_string(), neg_count));
            words.push(gerund_form(lemma));
            words
        }
        VerbForm::PastParticiple => {
            let mut words = Vec::new();
            words.extend(std::iter::repeat_n("not".to_string(), neg_count));
            words.push(past_participle(lemma));
            words
        }
    };

    parts.extend(render_vp_tail(predicate, subject_position));
    join_nonempty(parts)
}

fn render_vp(vp: &VerbPhrase, subject_position: SubjectPosition) -> String {
    let mut parts = Vec::new();

    if let Some(subject) = subject_position.surface_form() {
        parts.push(subject);
    }

    parts.push(base_form(vp.head().as_str()));
    parts.extend(render_complement_list(vp.complements()));
    parts.extend(render_phrase_list(vp.adjuncts()));
    join_nonempty(parts)
}

fn render_vp_tail(vp: &VerbPhrase, subject_position: SubjectPosition) -> Vec<String> {
    let mut parts = Vec::new();

    if let Some(subject) = subject_position.surface_form() {
        parts.push(subject);
    }

    parts.extend(render_complement_list(vp.complements()));
    parts.extend(render_phrase_list(vp.adjuncts()));
    parts
}

impl SubjectPosition {
    fn surface_form(self) -> Option<String> {
        match self {
            SubjectPosition::None | SubjectPosition::Trace => None,
        }
    }
}

fn is_pronoun_dp(dp: &DeterminerPhrase) -> bool {
    matches!(dp, DeterminerPhrase::Pronoun { .. })
}

fn render_possessor(dp: &DeterminerPhrase) -> String {
    let rendered = render_dp(dp, DpRenderRole::PossessiveDependent);
    if rendered.is_empty() {
        rendered
    } else if is_pronoun_dp(dp) {
        rendered
    } else {
        English::add_possessive(&rendered)
    }
}

fn render_dp(dp: &DeterminerPhrase, role: DpRenderRole) -> String {
    match dp {
        DeterminerPhrase::BareNominal(nominal) => render_nominal_dp(None, None, nominal),
        DeterminerPhrase::DeterminedNominal {
            determiner,
            nominal,
        } => render_nominal_dp(Some(*determiner), None, nominal),
        DeterminerPhrase::PossessedNominal { possessor, nominal } => {
            render_nominal_dp(None, Some(possessor), nominal)
        }
        DeterminerPhrase::ProperName(name) => name.clone(),
        DeterminerPhrase::Pronoun { pronoun, reflexive } => {
            render_pronoun(pronoun, *reflexive, role)
        }
    }
}

fn render_nominal_dp(
    determiner: Option<Determiner>,
    possessor: Option<&DeterminerPhrase>,
    nominal: &NounPhrase,
) -> String {
    let mut parts = Vec::new();

    if let Some(possessor) = possessor {
        let possessor = render_possessor(possessor);
        if !possessor.is_empty() {
            parts.push(possessor);
        }
    }

    let complement = render_np(nominal);

    if let Some(determiner) = determiner {
        let determiner = match determiner {
            Determiner::Indefinite => indefinite_article(&complement).to_string(),
            _ => determiner.as_str().to_string(),
        };
        parts.push(determiner);
    }

    if !complement.is_empty() {
        parts.push(complement);
    }

    join_nonempty(parts)
}

fn render_np(np: &NounPhrase) -> String {
    let mut parts = render_phrase_list(np.modifiers());
    parts.push(match np.head() {
        entry => English::noun(entry.as_str(), np.number()),
    });
    parts.extend(render_complement_list(np.complements()));
    parts.extend(render_phrase_list(np.adjuncts()));
    join_nonempty(parts)
}

fn render_pronoun(entry: &Pronoun, reflexive: bool, role: DpRenderRole) -> String {
    if reflexive {
        entry.reflexive_form().to_string()
    } else {
        match role {
            DpRenderRole::Subject => entry.subject_form().to_string(),
            DpRenderRole::Object => entry.object_form().to_string(),
            DpRenderRole::PossessiveDependent => entry.possessive_dependent_form().to_string(),
        }
    }
}

fn render_ap(ap: &AdjectivePhrase) -> String {
    let mut parts = Vec::new();
    if let Some(specifier) = ap.modifier_opt() {
        parts.push(render_advp(specifier));
    }
    parts.push(English::adj(ap.head().as_str(), &english::Degree::Positive));
    parts.extend(render_complement_list(ap.complements()));
    join_nonempty(parts)
}

fn render_advp(advp: &AdverbPhrase) -> String {
    let mut parts = Vec::new();
    if let Some(specifier) = advp.modifier_opt() {
        parts.push(render_advp(specifier));
    }
    parts.push(advp.head().as_str().to_string());
    parts.extend(render_phrase_list(advp.complements()));
    join_nonempty(parts)
}

fn render_pp(pp: &PrepositionalPhrase) -> String {
    join_nonempty(vec![
        pp.head().as_str().to_string(),
        render_complement(pp.complement()),
    ])
}

impl private::Sealed for Phrase {}
impl private::Sealed for DeterminerPhrase {}
impl private::Sealed for NominalDeterminerPhrase {}
impl private::Sealed for PronominalDeterminerPhrase {}
impl private::Sealed for NounPhrase {}
impl private::Sealed for AdjectivePhrase {}
impl private::Sealed for AdverbPhrase {}
impl private::Sealed for PrepositionalPhrase {}
impl private::Sealed for VerbPhrase {}
impl<Form: ClauseForm> private::Sealed for TensePhrase<Form> {}
impl private::Sealed for ComplementizerPhrase {}

impl Realizable for Phrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_phrase(self), options)
    }
}

impl Realizable for DeterminerPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_dp(self, DpRenderRole::Subject), options)
    }
}

impl Realizable for NominalDeterminerPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        let dp = DeterminerPhrase::from(self.clone());
        dp.realize_with(options)
    }
}

impl Realizable for PronominalDeterminerPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        let dp = DeterminerPhrase::from(*self);
        dp.realize_with(options)
    }
}

impl Realizable for NounPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_np(self), options)
    }
}

impl Realizable for AdjectivePhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_ap(self), options)
    }
}

impl Realizable for AdverbPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_advp(self), options)
    }
}

impl Realizable for PrepositionalPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_pp(self), options)
    }
}

impl Realizable for VerbPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_vp(self, SubjectPosition::None), options)
    }
}

impl<Form: ClauseForm> Realizable for TensePhrase<Form> {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_tp(self), options)
    }
}

impl Realizable for ComplementizerPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_cp(self), options)
    }
}
