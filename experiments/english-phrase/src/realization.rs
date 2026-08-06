use crate::lexical::{Determiner, Pronoun};
use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, AgreementMarker, Complementizer, ContentClause,
    DeterminerPhrase, NominalAgreementMarker, NominalCountabilityMarker, NominalDeterminerPhrase,
    NominalNumberMarker, NounPhrase, PredicateGap, PrepositionalPhrase, PronominalDeterminerPhrase,
    RelativeClause, RelativeTpGap, Relativizer, Tense, TensePhrase, TpForm, TpGap, VerbForm,
    VerbPhrase,
};
use english::{English, Form as MorphForm, Number, Person, Tense as MorphTense};

mod runtime;

use self::runtime::{
    LowerRuntime, RuntimeAdjectivePhrase, RuntimeAdverbPhrase, RuntimeAdvpComplement,
    RuntimeApComplement, RuntimeContentClause, RuntimeDeterminerPhrase,
    RuntimeDeterminerPhraseKind, RuntimeNounPhrase, RuntimeNpAdjunct, RuntimeNpComplement,
    RuntimeNpModifier, RuntimePpComplement, RuntimePrepositionalPhrase, RuntimeRelativeClause,
    RuntimeTensePhrase, RuntimeVerbPhrase, RuntimeVpAdjunct, RuntimeVpComplement,
};

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

fn render_np_modifier(modifier: &RuntimeNpModifier) -> String {
    match modifier {
        RuntimeNpModifier::Adj(ap) => render_ap(ap),
    }
}

fn render_np_complement(complement: &RuntimeNpComplement) -> String {
    match complement {
        RuntimeNpComplement::PP(pp) => render_pp(pp),
        RuntimeNpComplement::TP(tp) => render_tp(tp),
        RuntimeNpComplement::ContentClause(cp) => render_content_clause(cp),
    }
}

fn render_np_adjunct(adjunct: &RuntimeNpAdjunct) -> String {
    match adjunct {
        RuntimeNpAdjunct::PP(pp) => render_pp(pp),
        RuntimeNpAdjunct::RelativeClause(relative) => render_relative_clause(relative),
    }
}

fn render_ap_complement(complement: &RuntimeApComplement) -> String {
    match complement {
        RuntimeApComplement::PP(pp) => render_pp(pp),
        RuntimeApComplement::TP(tp) => render_tp(tp),
        RuntimeApComplement::ContentClause(cp) => render_content_clause(cp),
    }
}

fn render_advp_complement(complement: &RuntimeAdvpComplement) -> String {
    match complement {
        RuntimeAdvpComplement::PP(pp) => render_pp(pp),
    }
}

fn render_pp_complement(complement: &RuntimePpComplement) -> String {
    match complement {
        RuntimePpComplement::DP(dp) => render_dp(dp, DpRenderRole::Object),
        RuntimePpComplement::PP(pp) => render_pp(pp),
        RuntimePpComplement::TP(tp) => render_tp(tp),
        RuntimePpComplement::ContentClause(cp) => render_content_clause(cp),
    }
}

fn render_vp_complement(complement: &RuntimeVpComplement) -> String {
    match complement {
        RuntimeVpComplement::DP(dp) => render_dp(dp, DpRenderRole::Object),
        RuntimeVpComplement::PP(pp) => render_pp(pp),
        RuntimeVpComplement::AP(ap) => render_ap(ap),
        RuntimeVpComplement::ContentClause(cp) => render_content_clause(cp),
        RuntimeVpComplement::TP(tp) => render_tp(tp),
    }
}

fn render_vp_adjunct(adjunct: &RuntimeVpAdjunct) -> String {
    match adjunct {
        RuntimeVpAdjunct::PP(pp) => render_pp(pp),
        RuntimeVpAdjunct::AdvP(advp) => render_advp(advp),
    }
}

fn render_content_clause(cp: &RuntimeContentClause) -> String {
    let mut parts = Vec::new();

    if let Some(head) = render_complementizer(cp.head) {
        parts.push(head.to_string());
    }

    parts.push(render_tp(cp.complement.as_ref()));
    join_nonempty(parts)
}

fn render_relative_clause(cp: &RuntimeRelativeClause) -> String {
    let mut parts = Vec::new();

    if let Some(head) = render_relativizer(cp.head) {
        parts.push(head.to_string());
    }

    parts.push(render_tp(cp.complement.as_ref()));
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

fn render_relativizer(head: Relativizer) -> Option<&'static str> {
    match head {
        Relativizer::Null => None,
        Relativizer::That => Some("that"),
        Relativizer::Who => Some("who"),
        Relativizer::Which => Some("which"),
    }
}

fn render_tp(tp: &RuntimeTensePhrase) -> String {
    let surfaced_subject = tp
        .subject
        .as_ref()
        .map(|dp| render_dp(dp, DpRenderRole::Subject));

    let predicate = render_tense_head(&tp.predicate, tp.form, tp.negative, tp.agreement);

    join_nonempty(
        surfaced_subject
            .into_iter()
            .chain(std::iter::once(predicate)),
    )
}

fn render_tense_head(
    predicate: &RuntimeVerbPhrase,
    form: VerbForm,
    negative: bool,
    agreement: Option<(Person, Number)>,
) -> String {
    let lemma = predicate.head.as_str();
    let neg_count = usize::from(negative);
    let agreement = agreement.unwrap_or((Person::Third, Number::Singular));

    let mut parts = match form {
        VerbForm::Finite(tense) => {
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

    parts.extend(predicate.complements.iter().map(render_vp_complement));
    parts.extend(predicate.adjuncts.iter().map(render_vp_adjunct));
    join_nonempty(parts)
}

fn render_vp(vp: &RuntimeVerbPhrase) -> String {
    let mut parts = Vec::new();
    parts.push(base_form(vp.head.as_str()));
    parts.extend(vp.complements.iter().map(render_vp_complement));
    parts.extend(vp.adjuncts.iter().map(render_vp_adjunct));
    join_nonempty(parts)
}

fn is_pronoun_dp(dp: &RuntimeDeterminerPhrase) -> bool {
    matches!(&dp.kind, RuntimeDeterminerPhraseKind::Pronoun { .. })
}

fn render_possessor(dp: &RuntimeDeterminerPhrase) -> String {
    let rendered = render_dp(dp, DpRenderRole::PossessiveDependent);
    if rendered.is_empty() {
        rendered
    } else if is_pronoun_dp(dp) {
        rendered
    } else {
        English::add_possessive(&rendered)
    }
}

fn render_dp(dp: &RuntimeDeterminerPhrase, role: DpRenderRole) -> String {
    match &dp.kind {
        RuntimeDeterminerPhraseKind::BareNominal(nominal) => render_nominal_dp(None, None, nominal),
        RuntimeDeterminerPhraseKind::DeterminedNominal {
            determiner,
            nominal,
        } => render_nominal_dp(Some(*determiner), None, nominal),
        RuntimeDeterminerPhraseKind::PossessedNominal { possessor, nominal } => {
            render_nominal_dp(None, Some(possessor.as_ref()), nominal)
        }
        RuntimeDeterminerPhraseKind::ProperName(name) => name.clone(),
        RuntimeDeterminerPhraseKind::Pronoun { pronoun, reflexive } => {
            render_pronoun(pronoun, *reflexive, role)
        }
    }
}

fn render_nominal_dp(
    determiner: Option<Determiner>,
    possessor: Option<&RuntimeDeterminerPhrase>,
    nominal: &RuntimeNounPhrase,
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

fn render_np(np: &RuntimeNounPhrase) -> String {
    let mut parts: Vec<String> = np.modifiers.iter().map(render_np_modifier).collect();
    parts.push(English::noun(np.head.as_str(), &np.number));
    parts.extend(np.complements.iter().map(render_np_complement));
    parts.extend(np.adjuncts.iter().map(render_np_adjunct));
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

fn render_ap(ap: &RuntimeAdjectivePhrase) -> String {
    let mut parts = Vec::new();
    if let Some(specifier) = ap.modifier.as_deref() {
        parts.push(render_advp(specifier));
    }
    parts.push(English::adj(ap.head.as_str(), &english::Degree::Positive));
    parts.extend(ap.complements.iter().map(render_ap_complement));
    join_nonempty(parts)
}

fn render_advp(advp: &RuntimeAdverbPhrase) -> String {
    let mut parts = Vec::new();
    if let Some(specifier) = advp.modifier.as_deref() {
        parts.push(render_advp(specifier));
    }
    parts.push(advp.head.as_str().to_string());
    parts.extend(advp.complements.iter().map(render_advp_complement));
    join_nonempty(parts)
}

fn render_pp(pp: &RuntimePrepositionalPhrase) -> String {
    join_nonempty(vec![
        pp.head.as_str().to_string(),
        render_pp_complement(pp.complement.as_ref()),
    ])
}

impl<A: AgreementMarker> private::Sealed for DeterminerPhrase<A> {}
impl<N: NominalNumberMarker, C: NominalCountabilityMarker> private::Sealed
    for NominalDeterminerPhrase<N, C>
{
}
impl private::Sealed for PronominalDeterminerPhrase {}
impl<N: NominalNumberMarker, C: NominalCountabilityMarker> private::Sealed for NounPhrase<N, C> {}
impl private::Sealed for AdjectivePhrase {}
impl private::Sealed for AdverbPhrase {}
impl private::Sealed for PrepositionalPhrase {}
impl<G: PredicateGap> private::Sealed for VerbPhrase<G> {}
impl<Form: TpForm, G: TpGap, A: AgreementMarker> private::Sealed for TensePhrase<Form, G, A> {}
impl private::Sealed for ContentClause {}
impl<G: RelativeTpGap> private::Sealed for RelativeClause<G> where
    crate::syntax::RelativeForce: crate::syntax::CpForce<G, Head = Relativizer>
{
}

impl<A: AgreementMarker> Realizable for DeterminerPhrase<A> {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(
            render_dp(&self.lower_runtime(), DpRenderRole::Subject),
            options,
        )
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> Realizable
    for NominalDeterminerPhrase<N, C>
where
    N: NominalAgreementMarker,
{
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(
            render_dp(&self.lower_runtime(), DpRenderRole::Subject),
            options,
        )
    }
}

impl Realizable for PronominalDeterminerPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(
            render_dp(&self.lower_runtime(), DpRenderRole::Subject),
            options,
        )
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> Realizable for NounPhrase<N, C> {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_np(&self.lower_runtime()), options)
    }
}

impl Realizable for AdjectivePhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_ap(&self.lower_runtime()), options)
    }
}

impl Realizable for AdverbPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_advp(&self.lower_runtime()), options)
    }
}

impl Realizable for PrepositionalPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_pp(&self.lower_runtime()), options)
    }
}

impl<G: PredicateGap> Realizable for VerbPhrase<G> {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_vp(&self.lower_runtime()), options)
    }
}

impl<Form: TpForm, G: TpGap, A: AgreementMarker> Realizable for TensePhrase<Form, G, A> {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_tp(&self.lower_runtime()), options)
    }
}

impl Realizable for ContentClause {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_content_clause(&self.lower_runtime()), options)
    }
}

impl<G: RelativeTpGap> Realizable for RelativeClause<G>
where
    crate::syntax::RelativeForce: crate::syntax::CpForce<G, Head = Relativizer>,
{
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_relative_clause(&self.lower_runtime()), options)
    }
}
