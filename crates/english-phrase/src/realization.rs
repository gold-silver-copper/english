use crate::lexical::{Determiner, Pronoun};
use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, AdvpComplement, AgreementMarker, ApComplement, Complementizer,
    ContentClause, DeterminerPhrase, DeterminerPhraseKind, DynamicDeterminerPhrase,
    NominalCountabilityMarker, NominalDeterminerPhrase, NominalNumberMarker, NounPhrase,
    NounPhraseData, NpAdjunct, NpComplement, NpModifier, PpComplement, PredicateGap,
    PrepositionalPhrase, PronominalDeterminerPhrase, RelativeClause, RelativeTpGap, Relativizer,
    Tense, TensePhrase, TpForm, TpGap, VerbForm, VerbPhrase, VpAdjunct, VpArgumentSlot,
    VpComplement,
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

fn agreement_from_dp(dp: &DynamicDeterminerPhrase) -> (Person, Number) {
    match &dp.kind {
        DeterminerPhraseKind::BareNominal(nominal)
        | DeterminerPhraseKind::DeterminedNominal { nominal, .. }
        | DeterminerPhraseKind::PossessedNominal { nominal, .. } => {
            (Person::Third, nominal.number.clone())
        }
        DeterminerPhraseKind::ProperName(_) => (Person::Third, Number::Singular),
        DeterminerPhraseKind::Pronoun { pronoun, .. } => (pronoun.person(), pronoun.number()),
    }
}

fn render_np_modifier(modifier: &NpModifier) -> String {
    match modifier {
        NpModifier::Adj(ap) => render_ap(ap),
    }
}

fn render_np_complement(complement: &NpComplement) -> String {
    match complement {
        NpComplement::PP(pp) => render_pp(pp),
        NpComplement::ToInf(tp) => render_tp(tp),
        NpComplement::CP(cp) => render_cp(cp),
    }
}

fn render_np_adjunct(adjunct: &NpAdjunct) -> String {
    match adjunct {
        NpAdjunct::PP(pp) => render_pp(pp),
        NpAdjunct::RelativeObject(relative) => render_relative_cp(relative),
        NpAdjunct::RelativeSubjectSingular(relative) => render_relative_cp(relative),
        NpAdjunct::RelativeSubjectPlural(relative) => render_relative_cp(relative),
    }
}

fn render_ap_complement(complement: &ApComplement) -> String {
    match complement {
        ApComplement::PP(pp) => render_pp(pp),
        ApComplement::ToInf(tp) => render_tp(tp),
        ApComplement::CP(cp) => render_cp(cp),
    }
}

fn render_advp_complement(complement: &AdvpComplement) -> String {
    match complement {
        AdvpComplement::PP(pp) => render_pp(pp),
    }
}

fn render_pp_complement(complement: &PpComplement) -> String {
    match complement {
        PpComplement::DP(dp) => render_dp(dp, DpRenderRole::Object),
        PpComplement::PP(pp) => render_pp(pp),
        PpComplement::Gerund(tp) => render_tp(tp),
        PpComplement::CP(cp) => render_cp(cp),
    }
}

fn render_vp_complement(complement: &VpComplement) -> String {
    match complement {
        VpComplement::DP(dp) => render_dp(dp, DpRenderRole::Object),
        VpComplement::PP(pp) => render_pp(pp),
        VpComplement::AP(ap) => render_ap(ap),
        VpComplement::CP(cp) => render_cp(cp),
        VpComplement::BareInf(tp) => render_tp(tp),
        VpComplement::ToInf(tp) => render_tp(tp),
        VpComplement::Gerund(tp) => render_tp(tp),
        VpComplement::PastParticiple(tp) => render_tp(tp),
    }
}

fn render_vp_adjunct(adjunct: &VpAdjunct) -> String {
    match adjunct {
        VpAdjunct::PP(pp) => render_pp(pp),
        VpAdjunct::AdvP(advp) => render_advp(advp),
    }
}

fn render_cp(cp: &ContentClause) -> String {
    let mut parts = Vec::new();

    if let Some(head) = render_complementizer(cp.head()) {
        parts.push(head.to_string());
    }

    parts.push(render_tp(cp.complement()));
    join_nonempty(parts)
}

fn render_relative_cp<G: RelativeTpGap>(cp: &RelativeClause<G>) -> String
where
    crate::syntax::RelativeForce: crate::syntax::CpForce<G, Head = Relativizer>,
{
    let mut parts = Vec::new();

    if let Some(head) = render_relativizer(cp.head()) {
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

fn render_relativizer(head: Relativizer) -> Option<&'static str> {
    match head {
        Relativizer::Null => None,
        Relativizer::That => Some("that"),
        Relativizer::Who => Some("who"),
        Relativizer::Which => Some("which"),
    }
}

fn render_tp<Form: TpForm, G: TpGap, A: AgreementMarker>(tp: &TensePhrase<Form, G, A>) -> String {
    let surfaced_subject = tp
        .subject_opt()
        .map(|dp| render_dp(dp, DpRenderRole::Subject));

    let predicate = render_tense_head::<G, A>(
        tp.predicate(),
        tp.form(),
        tp.is_negative(),
        tp.subject_opt(),
    );

    join_nonempty(
        surfaced_subject
            .into_iter()
            .chain(std::iter::once(predicate)),
    )
}

fn render_tense_head<G: TpGap, A: AgreementMarker>(
    predicate: &VerbPhrase<G::PredicateGap>,
    form: VerbForm,
    negative: bool,
    subject: Option<&DynamicDeterminerPhrase>,
) -> String {
    let lemma = predicate.head().as_str();
    let neg_count = usize::from(negative);

    let mut parts = match form {
        VerbForm::Finite(tense) => {
            let agreement = A::agreement()
                .or_else(|| subject.map(agreement_from_dp))
                .or_else(G::subject_agreement)
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

    parts.extend(
        predicate
            .argument_slots()
            .iter()
            .map(render_vp_argument_slot)
            .filter(|part| !part.is_empty()),
    );
    parts.extend(predicate.adjuncts().iter().map(render_vp_adjunct));
    join_nonempty(parts)
}

fn render_vp<G: PredicateGap>(vp: &VerbPhrase<G>) -> String {
    let mut parts = Vec::new();
    parts.push(base_form(vp.head().as_str()));
    parts.extend(
        vp.argument_slots()
            .iter()
            .map(render_vp_argument_slot)
            .filter(|part| !part.is_empty()),
    );
    parts.extend(vp.adjuncts().iter().map(render_vp_adjunct));
    join_nonempty(parts)
}

fn render_vp_argument_slot(slot: &VpArgumentSlot) -> String {
    match slot {
        VpArgumentSlot::Complement(complement) => render_vp_complement(complement),
        VpArgumentSlot::GapObject => String::new(),
    }
}

fn is_pronoun_dp<A: AgreementMarker>(dp: &DeterminerPhrase<A>) -> bool {
    matches!(&dp.kind, DeterminerPhraseKind::Pronoun { .. })
}

fn render_possessor(dp: &DynamicDeterminerPhrase) -> String {
    let rendered = render_dp(dp, DpRenderRole::PossessiveDependent);
    if rendered.is_empty() {
        rendered
    } else if is_pronoun_dp(dp) {
        rendered
    } else {
        English::add_possessive(&rendered)
    }
}

fn render_dp<A: AgreementMarker>(dp: &DeterminerPhrase<A>, role: DpRenderRole) -> String {
    match &dp.kind {
        DeterminerPhraseKind::BareNominal(nominal) => render_nominal_dp(None, None, nominal),
        DeterminerPhraseKind::DeterminedNominal {
            determiner,
            nominal,
        } => render_nominal_dp(Some(*determiner), None, nominal),
        DeterminerPhraseKind::PossessedNominal { possessor, nominal } => {
            render_nominal_dp(None, Some(possessor), nominal)
        }
        DeterminerPhraseKind::ProperName(name) => name.clone(),
        DeterminerPhraseKind::Pronoun { pronoun, reflexive } => {
            render_pronoun(pronoun, *reflexive, role)
        }
    }
}

fn render_nominal_dp(
    determiner: Option<Determiner>,
    possessor: Option<&DynamicDeterminerPhrase>,
    nominal: &NounPhraseData,
) -> String {
    let mut parts = Vec::new();

    if let Some(possessor) = possessor {
        let possessor = render_possessor(possessor);
        if !possessor.is_empty() {
            parts.push(possessor);
        }
    }

    let complement = render_np_data(nominal);

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

fn render_np_data(np: &NounPhraseData) -> String {
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

fn render_ap(ap: &AdjectivePhrase) -> String {
    let mut parts = Vec::new();
    if let Some(specifier) = ap.modifier_opt() {
        parts.push(render_advp(specifier));
    }
    parts.push(English::adj(ap.head().as_str(), &english::Degree::Positive));
    parts.extend(ap.complements().iter().map(render_ap_complement));
    join_nonempty(parts)
}

fn render_advp(advp: &AdverbPhrase) -> String {
    let mut parts = Vec::new();
    if let Some(specifier) = advp.modifier_opt() {
        parts.push(render_advp(specifier));
    }
    parts.push(advp.head().as_str().to_string());
    parts.extend(advp.complements().iter().map(render_advp_complement));
    join_nonempty(parts)
}

fn render_pp(pp: &PrepositionalPhrase) -> String {
    join_nonempty(vec![
        pp.head().as_str().to_string(),
        render_pp_complement(pp.complement()),
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
        apply_realization_options(render_dp(self, DpRenderRole::Subject), options)
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> Realizable
    for NominalDeterminerPhrase<N, C>
where
    N: crate::syntax::NominalAgreementMarker,
{
    fn realize_with(&self, options: RealizationOptions) -> String {
        let dp: DynamicDeterminerPhrase =
            DeterminerPhrase::<N::Agreement>::from(self.clone()).erase();
        dp.realize_with(options)
    }
}

impl Realizable for PronominalDeterminerPhrase {
    fn realize_with(&self, options: RealizationOptions) -> String {
        let dp: DynamicDeterminerPhrase = (*self).into();
        dp.realize_with(options)
    }
}

impl<N: NominalNumberMarker, C: NominalCountabilityMarker> Realizable for NounPhrase<N, C> {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_np_data(&self.data), options)
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

impl<G: PredicateGap> Realizable for VerbPhrase<G> {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_vp(self), options)
    }
}

impl<Form: TpForm, G: TpGap, A: AgreementMarker> Realizable for TensePhrase<Form, G, A> {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_tp(self), options)
    }
}

impl Realizable for ContentClause {
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_cp(self), options)
    }
}

impl<G: RelativeTpGap> Realizable for RelativeClause<G>
where
    crate::syntax::RelativeForce: crate::syntax::CpForce<G, Head = Relativizer>,
{
    fn realize_with(&self, options: RealizationOptions) -> String {
        apply_realization_options(render_relative_cp(self), options)
    }
}
