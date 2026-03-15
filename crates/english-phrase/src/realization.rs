use crate::desugar::{
    lower_advp, lower_ap, lower_dp, lower_np, lower_phrase, lower_pp, lower_tense_phrase,
    lower_verb_projection,
};
use crate::error::RealizationResult;
use crate::internal::{
    ABar, AP, AdvBar, CBar, CP, DBar, DComplement, DHead, DP, NBar, NHead, NP, NegHead, NegVBar,
    PBar, PP, SilentDeterminer, TBar, THead, TP, VBar, VP, VPBar, XP,
};
use crate::syntax::{
    AdjectivePhrase, AdverbPhrase, DeterminerPhrase, NounPhrase, Phrase, PrepositionalPhrase,
    Sentence, Tense, TensePhrase, Terminal, VerbPhrase,
};
use english::{English, Form as MorphForm, Number, Person, Tense as MorphTense};
use std::borrow::Borrow;

mod private {
    pub trait Sealed {}
}

pub trait Realizable: private::Sealed {
    fn realize(&self) -> RealizationResult<String>;
}

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

fn render_xp_list(values: &[Box<XP>]) -> RealizationResult<Vec<String>> {
    values
        .iter()
        .map(|value| render_xp(value.as_ref()))
        .collect()
}

fn agreement_from_dp(dp: &DP) -> (Person, Number) {
    match &dp.bar.complement {
        DComplement::NP(np) => match &np.bar.head {
            NHead::CommonNoun { number, .. } => (Person::Third, number.clone()),
            NHead::ProperName(_) => (Person::Third, Number::Singular),
            NHead::Pronoun(pronoun) => (pronoun.person(), pronoun.number()),
        },
        DComplement::Trace => (Person::Third, Number::Singular),
    }
}

fn render_xp(xp: &XP) -> RealizationResult<String> {
    match xp {
        XP::TP(tp) => render_tp(tp),
        XP::VP(vp) => render_vp(vp),
        XP::DP(dp) => render_dp(dp),
        XP::NP(np) => render_np(np),
        XP::AP(ap) => render_ap(ap),
        XP::AdvP(advp) => render_advp(advp),
        XP::PP(pp) => render_pp(pp),
    }
}

fn render_cp(cp: &CP) -> RealizationResult<String> {
    render_cbar(&cp.bar)
}

fn render_cbar(cbar: &CBar) -> RealizationResult<String> {
    let _ = cbar.head;
    render_tp(&cbar.complement)
}

fn render_tp(tp: &TP) -> RealizationResult<String> {
    let subject = tp
        .specifier
        .as_ref()
        .map(|subject| render_dp(subject))
        .transpose()?;

    let predicate = render_tbar(&tp.bar, tp.specifier.as_deref())?;

    Ok(join_nonempty(
        subject.into_iter().chain(std::iter::once(predicate)),
    ))
}

fn render_tbar(tbar: &TBar, subject: Option<&DP>) -> RealizationResult<String> {
    let (neg_count, vp) = peel_negation(&tbar.complement);
    let vbar = headed_vbar(vp);
    let lemma = vbar.head.entry.as_str();
    let mut parts = match tbar.head {
        THead::Finite(tense) => {
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
        THead::BareInfinitive => {
            let mut words = Vec::new();
            words.extend(std::iter::repeat_n("not".to_string(), neg_count));
            words.push(base_form(lemma));
            words
        }
        THead::ToInfinitive => {
            let mut words = Vec::new();
            words.extend(std::iter::repeat_n("not".to_string(), neg_count));
            words.push("to".to_string());
            words.push(base_form(lemma));
            words
        }
        THead::GerundParticiple => {
            let mut words = Vec::new();
            words.extend(std::iter::repeat_n("not".to_string(), neg_count));
            words.push(gerund_form(lemma));
            words
        }
        THead::PastParticiple => {
            let mut words = Vec::new();
            words.extend(std::iter::repeat_n("not".to_string(), neg_count));
            words.push(past_participle(lemma));
            words
        }
    };

    parts.extend(render_vp_tail(vp)?);
    Ok(join_nonempty(parts))
}

fn peel_negation(mut vp: &VP) -> (usize, &VP) {
    let mut neg_count = 0;
    loop {
        match &vp.bar {
            VPBar::Negated(neg) => {
                neg_count += matches!(neg.head, NegHead::Not) as usize;
                vp = neg.complement.as_ref();
            }
            VPBar::Headed(_) => return (neg_count, vp),
        }
    }
}

fn headed_vbar(vp: &VP) -> &VBar {
    match &vp.bar {
        VPBar::Headed(vbar) => vbar,
        VPBar::Negated(_) => unreachable!("negation should be peeled before accessing V head"),
    }
}

fn render_vp(vp: &VP) -> RealizationResult<String> {
    match &vp.bar {
        VPBar::Headed(vbar) => render_vbar(vbar, vp.specifier.as_deref()),
        VPBar::Negated(negbar) => render_negated_vp(negbar, vp.specifier.as_deref()),
    }
}

fn render_negated_vp(negbar: &NegVBar, specifier: Option<&DP>) -> RealizationResult<String> {
    let mut parts = Vec::new();

    if let Some(specifier) = specifier {
        parts.push(render_dp(specifier)?);
    }

    parts.push(match negbar.head {
        NegHead::Not => "not".to_string(),
    });
    parts.push(render_vp(&negbar.complement)?);
    Ok(join_nonempty(parts))
}

fn render_vbar(vbar: &VBar, specifier: Option<&DP>) -> RealizationResult<String> {
    let head = base_form(vbar.head.entry.as_str());
    let mut parts = Vec::new();

    if let Some(specifier) = specifier {
        parts.push(render_dp(specifier)?);
    }

    parts.push(head);
    parts.extend(render_xp_list(&vbar.complements)?);
    parts.extend(render_xp_list(&vbar.adjuncts)?);
    Ok(join_nonempty(parts))
}

fn render_vp_tail(vp: &VP) -> RealizationResult<Vec<String>> {
    let vbar = headed_vbar(vp);
    let mut parts = Vec::new();

    if let Some(specifier) = &vp.specifier {
        let rendered = render_dp(specifier)?;
        if !rendered.is_empty() {
            parts.push(rendered);
        }
    }

    parts.extend(render_xp_list(&vbar.complements)?);
    parts.extend(render_xp_list(&vbar.adjuncts)?);
    Ok(parts)
}

fn render_dp(dp: &DP) -> RealizationResult<String> {
    render_dbar(&dp.bar)
}

fn render_dbar(dbar: &DBar) -> RealizationResult<String> {
    match (&dbar.head, &dbar.complement) {
        (DHead::Overt(head), DComplement::NP(np)) => Ok(join_nonempty(vec![
            head.as_str().to_string(),
            render_np(np)?,
        ])),
        (DHead::Silent(SilentDeterminer::Trace), DComplement::Trace) => Ok(String::new()),
        (DHead::Silent(_), DComplement::NP(np)) => render_np(np),
        (_, DComplement::Trace) => Ok(String::new()),
    }
}

fn render_np(np: &NP) -> RealizationResult<String> {
    let mut parts = render_xp_list(&np.left_adjuncts)?;
    parts.push(render_nbar(&np.bar)?);
    Ok(join_nonempty(parts))
}

fn render_nbar(nbar: &NBar) -> RealizationResult<String> {
    let head = match &nbar.head {
        NHead::CommonNoun { entry, number } => English::noun(entry.as_str(), number),
        NHead::ProperName(name) => name.clone(),
        NHead::Pronoun(pronoun) => pronoun.as_str().to_string(),
    };

    Ok(join_nonempty(
        std::iter::once(head).chain(render_xp_list(&nbar.complements)?),
    ))
}

fn render_ap(ap: &AP) -> RealizationResult<String> {
    let mut parts = Vec::new();
    if let Some(specifier) = &ap.specifier {
        parts.push(render_advp(specifier)?);
    }
    parts.push(render_abar(&ap.bar)?);
    Ok(join_nonempty(parts))
}

fn render_abar(abar: &ABar) -> RealizationResult<String> {
    Ok(join_nonempty(
        std::iter::once(English::adj(
            abar.head.entry.as_str(),
            &english::Degree::Positive,
        ))
        .chain(render_xp_list(&abar.complements)?),
    ))
}

fn render_advp(advp: &crate::internal::AdvP) -> RealizationResult<String> {
    let mut parts = Vec::new();
    if let Some(specifier) = &advp.specifier {
        parts.push(render_advp(specifier)?);
    }
    parts.push(render_advbar(&advp.bar)?);
    Ok(join_nonempty(parts))
}

fn render_advbar(advbar: &AdvBar) -> RealizationResult<String> {
    Ok(join_nonempty(
        std::iter::once(advbar.head.entry.as_str().to_string())
            .chain(render_xp_list(&advbar.complements)?),
    ))
}

fn render_pp(pp: &PP) -> RealizationResult<String> {
    render_pbar(&pp.bar)
}

fn render_pbar(pbar: &PBar) -> RealizationResult<String> {
    Ok(join_nonempty(vec![
        pbar.head.entry.as_str().to_string(),
        render_xp(pbar.complement.as_ref())?,
    ]))
}

pub fn realize_phrase(phrase: impl Borrow<Phrase>) -> RealizationResult<String> {
    phrase.borrow().realize()
}

pub fn realize_determiner_phrase(
    phrase: impl Borrow<DeterminerPhrase>,
) -> RealizationResult<String> {
    phrase.borrow().realize()
}

pub fn realize_noun_phrase(phrase: impl Borrow<NounPhrase>) -> RealizationResult<String> {
    phrase.borrow().realize()
}

pub fn realize_adjective_phrase(phrase: impl Borrow<AdjectivePhrase>) -> RealizationResult<String> {
    phrase.borrow().realize()
}

pub fn realize_adverb_phrase(phrase: impl Borrow<AdverbPhrase>) -> RealizationResult<String> {
    phrase.borrow().realize()
}

pub fn realize_prepositional_phrase(
    phrase: impl Borrow<PrepositionalPhrase>,
) -> RealizationResult<String> {
    phrase.borrow().realize()
}

pub fn realize_verb_phrase(phrase: impl Borrow<VerbPhrase>) -> RealizationResult<String> {
    phrase.borrow().realize()
}

pub fn realize_tense_phrase(phrase: impl Borrow<TensePhrase>) -> RealizationResult<String> {
    phrase.borrow().realize()
}

pub fn realize_sentence(sentence: impl Borrow<Sentence>) -> RealizationResult<String> {
    sentence.borrow().realize()
}

impl private::Sealed for Phrase {}
impl private::Sealed for DeterminerPhrase {}
impl private::Sealed for NounPhrase {}
impl private::Sealed for AdjectivePhrase {}
impl private::Sealed for AdverbPhrase {}
impl private::Sealed for PrepositionalPhrase {}
impl private::Sealed for VerbPhrase {}
impl private::Sealed for TensePhrase {}
impl private::Sealed for Sentence {}

impl Realizable for Phrase {
    fn realize(&self) -> RealizationResult<String> {
        render_xp(&lower_phrase(self)?)
    }
}

impl Realizable for DeterminerPhrase {
    fn realize(&self) -> RealizationResult<String> {
        render_dp(&lower_dp(self)?)
    }
}

impl Realizable for NounPhrase {
    fn realize(&self) -> RealizationResult<String> {
        render_np(&lower_np(self)?)
    }
}

impl Realizable for AdjectivePhrase {
    fn realize(&self) -> RealizationResult<String> {
        render_ap(&lower_ap(self)?)
    }
}

impl Realizable for AdverbPhrase {
    fn realize(&self) -> RealizationResult<String> {
        render_advp(&lower_advp(self)?)
    }
}

impl Realizable for PrepositionalPhrase {
    fn realize(&self) -> RealizationResult<String> {
        render_pp(&lower_pp(self)?)
    }
}

impl Realizable for VerbPhrase {
    fn realize(&self) -> RealizationResult<String> {
        render_tp(&lower_verb_projection(self, None)?)
    }
}

impl Realizable for TensePhrase {
    fn realize(&self) -> RealizationResult<String> {
        render_cp(&lower_tense_phrase(self)?)
    }
}

impl Realizable for Sentence {
    fn realize(&self) -> RealizationResult<String> {
        let mut text = self.tense_phrase().realize()?;
        if self.capitalize_flag() {
            text = English::capitalize_first(&text);
        }
        text.push(match self.terminal() {
            Terminal::Period => '.',
            Terminal::QuestionMark => '?',
            Terminal::ExclamationMark => '!',
        });
        Ok(text)
    }
}
