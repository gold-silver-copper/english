//! Dump scanning and candidate policy — the editorial layer of the pipeline.
//!
//! All three part-of-speech extractors share one shape:
//!
//! 1. stream the filtered dump, one JSON entry per line, keeping only proper
//!    entries ([`crate::helpers::entry_is_proper`]);
//! 2. gather the entry's eligible forms per inflection slot IN LISTING ORDER
//!    (order is editorial data: head templates put the canonical form first);
//! 3. choose one CANONICAL pattern per entry ([`crate::helpers::choose_form`])
//!    and emit every remaining plain attested pattern as an additional
//!    candidate — each variant becomes its own `_<n>` key downstream;
//! 4. per lemma, drop the candidate that equals the regular-rule prediction
//!    ([`LemmaAcc::drop_regular`] — the rule engine serves it at runtime) and
//!    hand the survivors to [`Lock::resolve`] for stable key assignment;
//! 5. [`Lock::reap`] lemmas absent from the dump, with a mass-absence guard
//!    whose skip is a HARD ERROR upstream (see `pipeline`) — a truncated dump
//!    must never silently produce a "successful" refresh.
//!
//! The policy invariants (each pinned by a fixture test at the bottom):
//! - only PLAIN forms are ever emitted; soft-tagged variants
//!   (nonstandard/dialectal/...) never become output, and soft-SENSE entries
//!   never demote a standard sibling off the bare key;
//! - a plainly-attested regular reserves the bare key for the rule engine, so
//!   bare keys degrade to the rule rather than to an arbitrary irregular;
//! - periphrastic adjective attestations are admissible ("more fun",
//!   "farther downhill", "better known") but only CANONICAL when no
//!   single-word plain form exists for the slot.

use crate::helpers::{
    Entry, Pos, choose_form, entry_is_proper, entry_is_soft, suffix_rule, tag_rank, word_is_proper,
};
use crate::registry::{Candidate, Lock, ReapReport, forms_sig, split_key};
use csv::{ReaderBuilder, WriterBuilder};
use english_core::*;
use std::collections::{BTreeMap, HashMap, HashSet};

/// A refresh that would retire more than this fraction of a part-of-speech's active
/// lock rows as "absent" is treated as a bad/partial dump: reaping is skipped and
/// reported rather than silently gutting the lockfile. Legitimate cross-dump churn
/// is far below this.
const REAP_MAX_FRACTION: f64 = 0.10;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Per-lemma accumulator while scanning the dump: distinct inflection patterns
/// (keyed by their form signature so duplicates merge) plus whether a
/// regular-prediction-equal pattern was seen (and dropped as a runtime fall-through).
#[derive(Default)]
struct LemmaAcc {
    /// emitted-form signature -> candidate (anchors enriched across the entries that
    /// share it). Keyed by `registry::forms_sig` (the same encoding `Candidate::sig`
    /// and `drop_regular` use), so two entries merge only when they emit identical
    /// forms.
    by_sig: BTreeMap<String, Candidate>,
    had_regular: bool,
}

impl LemmaAcc {
    /// Record one observed inflection pattern (the plain-preferred emitted columns)
    /// for this lemma, attaching the entry's stable anchors and its listing
    /// `ordinal` (0 = the entry's canonical pattern, higher = alternate variants,
    /// used only as the cold-start numbering tiebreak). Deduped by the emitted
    /// signature, so two entries merge only when they emit the same forms.
    fn observe(&mut self, forms: Vec<String>, ordinal: u8, entry: &Entry) {
        let soft = entry_is_soft(entry);
        let sig = forms_sig(&forms);
        let is_new = !self.by_sig.contains_key(&sig);
        let c = self
            .by_sig
            .entry(sig)
            .or_insert_with(|| Candidate::new(forms));
        c.ordinal = if is_new {
            ordinal
        } else {
            c.ordinal.min(ordinal)
        };
        // Soft only while EVERY contributing entry is soft: one standard sense
        // sharing the signature makes the candidate standard.
        c.soft_sense = if is_new { soft } else { c.soft_sense && soft };
        // Anchor fields merge as the MINIMUM across all contributing entries —
        // never first-writer-wins, which would make the selected anchor depend
        // on upstream entry order (a reorder could then change a new row's
        // frozen identity). min() is order-independent and matches the
        // "lowest/stablest" convention of Entry::lowest_qid.
        merge_min(&mut c.qid, entry.lowest_qid());
        merge_min(&mut c.sid, entry.lowest_senseid());
        merge_min(&mut c.etym, entry.etymology_number);
        // The gloss is only a human review aid, but it is PERSISTED in the
        // lockfile — first-writer-wins would make the committed CSV depend on
        // dump entry order, producing spurious lock diffs on an upstream
        // reorder. min() keeps it order-independent like the anchors.
        merge_min(&mut c.gloss, entry.first_gloss());
    }

    /// Drop the pattern that exactly equals the regular prediction; the rule engine
    /// produces it at runtime, so a table row would add nothing — PROVIDED the rule
    /// is actually reachable for those forms.
    ///
    /// `soft_reserves` decides how a rule-equal candidate whose senses are ALL soft
    /// is treated:
    /// - nouns and verbs pass `true`: any rule-equal candidate is dropped and
    ///   reserves the bare key (`had_regular`) — their predictions are the
    ///   near-universal defaults, and reserving keeps every attested variant
    ///   reachable (bare -> regular via rule, irregulars at numbered keys).
    ///   Measured on the real dump, the stricter guard cost nouns 1.5k+ reachable
    ///   attested plurals for zero accuracy gain.
    /// - adjectives pass `false`: a standard-sense rule-equal candidate drops and
    ///   reserves as usual, but a SOFT-sense one (the slang `bad` -> `badder`,
    ///   which the CVC rule predicts) must not demote a standard suppletive
    ///   sibling (`worse`) off the bare key. And since that sibling then OCCUPIES
    ///   the bare key, the rule never fires for this lemma — so the soft
    ///   candidate's attested forms would be unreachable if dropped. It is
    ///   therefore KEPT and published at a numbered key, unless it is the lemma's
    ///   sole candidate (then the bare key does fall through to the rule and the
    ///   row really would be redundant).
    fn drop_regular(&mut self, predicted: &[String], soft_reserves: bool) {
        let sig = forms_sig(predicted);
        let Some(c) = self.by_sig.get(&sig) else {
            return;
        };
        if soft_reserves || !c.soft_sense {
            self.by_sig.remove(&sig);
            self.had_regular = true;
            return;
        }
        // Soft rule-equal candidate. Reserving is only harmful when it would
        // demote a STANDARD sibling off the bare key (bad: worse must keep it).
        // With no standard sibling there is nothing to demote: reserve the bare
        // key for the rule (identical output, no redundant bare row) and let the
        // remaining soft variants take numbered keys.
        let others_all_soft = self
            .by_sig
            .iter()
            .filter(|(s, _)| **s != sig)
            .all(|(_, other)| other.soft_sense);
        if others_all_soft {
            self.by_sig.remove(&sig);
            self.had_regular = true;
        }
        // else: keep — the standard sibling will hold the bare key, so the rule
        // can never serve these attested forms; publish them at a numbered key.
    }

    fn into_candidates(self) -> (Vec<Candidate>, bool) {
        (self.by_sig.into_values().collect(), self.had_regular)
    }
}

/// Keep the smaller of the existing and incoming value (absent = no opinion).
/// Order-independent by construction: any permutation of contributors yields
/// the same result, unlike set-if-None.
fn merge_min<T: Ord>(slot: &mut Option<T>, incoming: Option<T>) {
    if let Some(new) = incoming {
        match slot {
            Some(cur) if *cur <= new => {}
            _ => *slot = Some(new),
        }
    }
}

fn open_reader(path: &Path) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

/// A dump pass may tolerate at most this fraction of JSONL lines failing to
/// deserialize. A healthy dump parses ~100%; a spike means wiktextract changed
/// its schema (it has before — see `de_etym_number` in helpers.rs), and the
/// affected lemmas would otherwise silently vanish from the scan and be reaped
/// as "absent from dump" with nothing pointing at the real cause.
const MAX_PARSE_FAILURE_FRACTION: f64 = 0.001;

/// Counts parse failures across one dump pass so they are REPORTED, never
/// silent: below the threshold a summary line still prints (the reap notes
/// misattribute such losses to absence otherwise), above it the pass hard-errors.
#[derive(Default)]
struct ParseTally {
    lines: usize,
    failures: usize,
}

impl ParseTally {
    fn parse(&mut self, line: &str) -> Option<Entry> {
        self.lines += 1;
        match serde_json::from_str(line) {
            Ok(entry) => Some(entry),
            Err(_) => {
                self.failures += 1;
                None
            }
        }
    }

    fn finish(self, pass: &str) -> Result<(), Box<dyn Error>> {
        if self.failures == 0 {
            return Ok(());
        }
        let fraction = self.failures as f64 / self.lines.max(1) as f64;
        if fraction > MAX_PARSE_FAILURE_FRACTION {
            return Err(format!(
                "{pass}: {} of {} lines ({:.2}%) failed to deserialize — likely a wiktextract \
                 schema change; refusing to continue (the affected lemmas would be silently \
                 reaped as absent). Fix the Entry schema in helpers.rs first.",
                self.failures,
                self.lines,
                100.0 * fraction
            )
            .into());
        }
        println!(
            "WARNING {pass}: {} of {} lines failed to deserialize (below the {:.2}% abort \
             threshold); their lemmas are invisible to this pass.",
            self.failures,
            self.lines,
            100.0 * MAX_PARSE_FAILURE_FRACTION
        );
        Ok(())
    }
}

/// Pick the form to emit for one inflection slot, falling back to the regular rule
/// prediction when no plain candidate is eligible.
fn select_slot(forms: &[(String, u8)], predicted: &str, prefer_regular: bool) -> String {
    choose_form(forms, predicted, prefer_regular).unwrap_or_else(|| predicted.to_string())
}

/// The plain attested alternates for one slot: everything except the chosen
/// canonical, deduped, in listing order. These become the k-th variant tuples.
fn slot_alternates(pool: &[(String, u8)], chosen: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (form, rank) in pool {
        if *rank == 0 && form != chosen && !out.iter().any(|f| f == form) {
            out.push(form.clone());
        }
    }
    out
}

/// A periphrastic attestation ("more X"/"most X") is only credible when nothing
/// single-word is plainly attested for the slot. When both appear, the suffixal
/// form is the canonical one (`badder` beats "more bad", `cleverer` beats "more
/// clever") — otherwise the anti-prediction preference in `choose_form` would
/// canonicalize the periphrastic variant for every word whose suffixal form the
/// rule engine predicts.
fn drop_periphrastic_if_single_word_attested(forms: &mut Vec<(String, u8)>) {
    if forms.iter().any(|(f, r)| *r == 0 && !f.contains(' ')) {
        forms.retain(|(f, _)| !f.contains(' '));
    }
}

// NOTE on uncountable nouns (investigated 2026-07, deliberately NOT implemented):
// marking "strictly uncountable" lemmas (every sense tagged `uncountable`, no
// plain plural) as invariant plurals sounds like it should fix
// `noun("information") -> "informations"`. Measured against the real dump it does
// the opposite: the common mass nouns (information, advice, rice, ...) all carry
// plainly-attested plurals and `countable` senses in Wiktionary, so no tag
// heuristic can qualify them — while ~85k tail-jargon entries (chemical names,
// -isms) DO qualify, quintupling the noun table for words nobody inflects.
// Treating those attested plurals as correct (Wiktionary itself says `advices`
// exists) is the more defensible behavior; a curated mass-noun list would be the
// way to change it, not dump tags.

/// Scan noun entries and resolve their plural patterns into `lock`.
///
/// Nouns have a single inflection slot, so every distinct plain attested
/// plural is simply its own candidate (no tuple alignment needed). Returns the
/// reap report — the caller MUST honor `skipped` (see the module docs).
pub fn extract_irregular_nouns(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
    date: &str,
) -> Result<ReapReport, Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let mut by_lemma: BTreeMap<String, LemmaAcc> = BTreeMap::new();

    let reader = open_reader(input_path);
    let mut tally = ParseTally::default();
    for line in reader.lines() {
        let line = line?;
        let Some(entry) = tally.parse(&line) else {
            continue;
        };

        if !entry_is_proper(&entry, "noun") {
            continue;
        }

        let lemma = entry.word.to_lowercase();

        // Gather every eligible plural-tagged form for THIS entry, in listing order.
        let mut plurals: Vec<(String, u8)> = Vec::new();
        if let Some(forms) = &entry.forms {
            for form in forms {
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" || !word_is_proper(&entry_form) {
                    continue;
                }
                if !form.tags.iter().any(|t| t == "plural") {
                    continue;
                }
                let Some(rank) = tag_rank(&form.tags) else {
                    continue;
                };
                plurals.push((entry_form, rank));
            }
        }

        if plurals.is_empty() {
            continue;
        }

        // Every distinct PLAIN attested plural becomes its own candidate: each
        // variant gets its own numbered key (`cactus_2` -> cacti, `cactus_3` -> ...),
        // with the listing index as the cold-start ordinal so the head template's
        // canonical-first order decides the numbering, not the alphabet. Soft
        // forms (`busses`, nonstandard `sheeps`) are never emitted. The candidate
        // that EQUALS the rule prediction is observed too and removed at lemma
        // level by drop_regular, which reserves the bare key for the rule engine
        // (`die` -> dies via the rule, `die_2` -> dice); only when no plain regular
        // is attested does an irregular take the bare key (`child` -> children,
        // `sheep` -> sheep). soft_reserves=true for nouns: a soft-SENSE entry's
        // plain regular still reserves — measured on the real dump, the
        // adjective-style guard cost 1.5k+ reachable attested plurals here.
        let acc = by_lemma.entry(lemma).or_default();
        let mut seen: Vec<&str> = Vec::new();
        for (form, rank) in &plurals {
            if *rank != 0 || seen.iter().any(|f| f == form) {
                continue;
            }
            seen.push(form);
            acc.observe(vec![form.clone()], (seen.len() - 1) as u8, &entry);
        }
    }
    tally.finish("extract_irregular_nouns")?;

    let mut resolved: HashSet<String> = HashSet::new();
    for (lemma, mut acc) in by_lemma {
        let predicted_plural = EnglishCore::pluralize_noun(&lemma);
        acc.drop_regular(&[predicted_plural], true);
        let (candidates, had_regular) = acc.into_candidates();
        if candidates.is_empty() {
            continue;
        }
        lock.resolve(&lemma, Pos::Noun, candidates, had_regular, date)?;
        resolved.insert(lemma);
    }
    let reap = lock.reap(Pos::Noun, &resolved, date, REAP_MAX_FRACTION);

    println!("Resolved irregular nouns into the lock.");
    Ok(reap)
}

/// Scan adjective entries and resolve their (comparative, superlative)
/// patterns into `lock`.
///
/// Adjectives carry the subtlest policy in the extractor because the rule
/// engine's prediction is linguistically informed (suffixal for short words,
/// periphrastic otherwise) and Wiktionary often attests BOTH shapes — see the
/// inline comments at the periphrastic filter and `drop_regular` call for the
/// full decision table. Returns the reap report; `skipped` must be honored.
pub fn extract_irregular_adjectives(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
    date: &str,
) -> Result<ReapReport, Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let mut by_lemma: BTreeMap<String, LemmaAcc> = BTreeMap::new();
    let reader = open_reader(input_path);

    let mut tally = ParseTally::default();
    for line in reader.lines() {
        let line = line?;
        let Some(entry) = tally.parse(&line) else {
            continue;
        };
        if !entry_is_proper(&entry, "adj") {
            continue;
        }

        let lemma = entry.word.to_lowercase();

        let mut comp_forms: Vec<(String, u8)> = Vec::new();
        let mut sup_forms: Vec<(String, u8)> = Vec::new();
        // Periphrastic attestations are multiword ("more fun") and would fail
        // word_is_proper; they are legitimate — and load-bearing now that the rule
        // engine predicts suffixal forms for short adjectives ("funner"): the
        // attested periphrastic must survive as a tabled exception. Locational
        // adjectives (downhill, downstream) inflect with farther/further, which
        // must be admitted too or those entries look formless and fall through to
        // a wrong "more downhill" from the rule.
        // Participial adjectives inflect with better/best ("better known").
        // NOT admitted: dialectal degree words ("mair reet" — Scots), the one
        // known residual class; see the accuracy harness docs.
        let periphrastic = [
            format!("more {lemma}"),
            format!("most {lemma}"),
            format!("farther {lemma}"),
            format!("further {lemma}"),
            format!("farthest {lemma}"),
            format!("furthest {lemma}"),
            format!("better {lemma}"),
            format!("best {lemma}"),
        ];
        if let Some(forms) = &entry.forms {
            for form in forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious"
                    || (!word_is_proper(&entry_form) && !periphrastic.contains(&entry_form))
                {
                    continue;
                }
                let Some(rank) = tag_rank(tags) else {
                    continue;
                };
                if tags.iter().any(|t| t == "comparative") {
                    comp_forms.push((entry_form.clone(), rank));
                }
                if tags.iter().any(|t| t == "superlative") {
                    sup_forms.push((entry_form, rank));
                }
            }
        }

        // Snapshot the full plain pools BEFORE the periphrastic-credibility filter:
        // the filter decides only which form is CANONICAL (freer beats "more free"
        // for the lowest key) — a filtered periphrastic is still a plain attested
        // variant and gets its own numbered key (`free2` -> "more free"/"most free").
        let all_comp = comp_forms.clone();
        let all_sup = sup_forms.clone();
        drop_periphrastic_if_single_word_attested(&mut comp_forms);
        drop_periphrastic_if_single_word_attested(&mut sup_forms);

        // An entry with NO plain-attested form in either slot (an uncomparable
        // sense, or one attesting only soft-tagged variants) contributes nothing:
        // observing it would fabricate a fully-predicted candidate, which
        // drop_regular then counts as a regular attestation — reserving the bare
        // key and demoting a genuine irregular sibling to suffix 2 (`fast` used to
        // emit "more fast" while faster/fastest sat at `fast2`). Reservation must
        // come from attested plain forms (`fun` really lists "more fun"), never
        // from a synthesized prediction.
        let attests_plain = comp_forms
            .iter()
            .chain(sup_forms.iter())
            .any(|(_, rank)| *rank == 0);
        if !attests_plain {
            continue;
        }

        let predicted_comp = EnglishCore::comparative(&lemma);
        let predicted_sup = EnglishCore::superlative(&lemma);
        let comparative = select_slot(&comp_forms, &predicted_comp, false);
        let superlative = select_slot(&sup_forms, &predicted_sup, false);

        // The canonical pattern plus one candidate per alternate rank: attested
        // variants get their own numbered keys, alternates aligned by rank across
        // the two slots (a slot without a k-th alternate keeps its canonical form).
        // Alternates come from the UNFILTERED pools so filtered periphrastics are
        // still published. Consequence handled deliberately: for lemmas whose rule
        // prediction is periphrastic (clever, simple), the re-admitted "more X"
        // variant tuple equals the prediction, is dropped by drop_regular, and —
        // when standard-sense — reserves the bare key for the rule engine, moving
        // the suffixal form to suffix 2. That is the same attested-regular-reserves
        // policy as die/die_2 and abide/abide_2.
        let acc = by_lemma.entry(lemma).or_default();
        acc.observe(vec![comparative.clone(), superlative.clone()], 0, &entry);
        let alt = [
            slot_alternates(&all_comp, &comparative),
            slot_alternates(&all_sup, &superlative),
        ];
        for k in 0..alt.iter().map(|v| v.len()).max().unwrap_or(0) {
            acc.observe(
                vec![
                    alt[0]
                        .get(k)
                        .cloned()
                        .unwrap_or_else(|| comparative.clone()),
                    alt[1]
                        .get(k)
                        .cloned()
                        .unwrap_or_else(|| superlative.clone()),
                ],
                (k + 1).min(u8::MAX as usize) as u8,
                &entry,
            );
        }
    }
    tally.finish("extract_irregular_adjectives")?;

    let mut resolved: HashSet<String> = HashSet::new();
    for (lemma, mut acc) in by_lemma {
        let predicted = [
            EnglishCore::comparative(&lemma),
            EnglishCore::superlative(&lemma),
        ];
        acc.drop_regular(&predicted, false);
        let (candidates, had_regular) = acc.into_candidates();
        if candidates.is_empty() {
            continue;
        }
        lock.resolve(&lemma, Pos::Adj, candidates, had_regular, date)?;
        resolved.insert(lemma);
    }
    let reap = lock.reap(Pos::Adj, &resolved, date, REAP_MAX_FRACTION);

    println!("Resolved irregular adjectives into the lock.");
    Ok(reap)
}

/// Scan verb entries and resolve their (3sg, past, present participle, past
/// participle) tuples into `lock`.
///
/// Missing slots are synthesized from the rule prediction (an entry attesting
/// only an irregular past must not vanish because the dump omitted its 3sg),
/// EXCEPT for entries attesting nothing at all, which are skipped outright —
/// a fully synthesized tuple would masquerade as a regular attestation and
/// wrongly reserve the bare key. `be` is skipped entirely: its suppletive
/// paradigm lives in `english-core`. Returns the reap report; `skipped` must
/// be honored.
pub fn extract_verb_conjugations(
    input_path: impl AsRef<Path>,
    lock: &mut Lock,
    date: &str,
) -> Result<ReapReport, Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let mut by_lemma: BTreeMap<String, LemmaAcc> = BTreeMap::new();
    let reader = open_reader(input_path);

    let mut tally = ParseTally::default();
    for line in reader.lines() {
        let line = line?;
        let Some(entry) = tally.parse(&line) else {
            continue;
        };
        if !entry_is_proper(&entry, "verb") {
            continue;
        }

        let lemma = entry.word.to_lowercase();
        // "to be" has too many forms for the (3sg, past, pres-part, past-part)
        // shape and is handled directly by english-core.
        if lemma == "be" {
            continue;
        }

        // Gather every eligible form per slot, then pick each deterministically.
        let mut third_forms: Vec<(String, u8)> = Vec::new();
        let mut past_forms: Vec<(String, u8)> = Vec::new();
        let mut pres_part_forms: Vec<(String, u8)> = Vec::new();
        let mut past_part_forms: Vec<(String, u8)> = Vec::new();

        if let Some(forms) = &entry.forms {
            for form in forms {
                let tags = &form.tags;
                let entry_form = form.form.to_lowercase();
                if entry_form == "dubious" || !word_is_proper(&entry_form) {
                    continue;
                }
                let Some(rank) = tag_rank(tags) else {
                    continue;
                };

                let has = |t: &str| tags.iter().any(|x| x == t);
                if has("third-person") && has("singular") && has("present") {
                    third_forms.push((entry_form.clone(), rank));
                }
                if has("past") && !has("participle") {
                    past_forms.push((entry_form.clone(), rank));
                }
                if has("participle") && has("present") {
                    pres_part_forms.push((entry_form.clone(), rank));
                }
                if has("participle") && has("past") {
                    past_part_forms.push((entry_form, rank));
                }
            }
        }

        // Same rule as adjectives: an entry with NO plain-attested form in ANY slot
        // must not fabricate a fully-predicted candidate (which drop_regular would
        // count as a regular attestation, reserving the bare key against a genuine
        // irregular sibling). The slot synthesis below is for entries that attest
        // SOME slots — e.g. an irregular past with the 3sg omitted — not for
        // entries that attest nothing at all.
        let attests_plain = [
            &third_forms,
            &past_forms,
            &pres_part_forms,
            &past_part_forms,
        ]
        .iter()
        .any(|slot| slot.iter().any(|(_, rank)| *rank == 0));
        if !attests_plain {
            continue;
        }

        let predicted_third = EnglishCore::verb(
            &lemma,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
        );
        let predicted_past = EnglishCore::verb(
            &lemma,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite,
        );
        let predicted_participle = EnglishCore::verb(
            &lemma,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle,
        );

        // Synthesize the regular 3sg when the dump omits it, exactly as we already do
        // for past/participles. A verb's presence must depend only on it being an
        // English verb, never on whether the dump happened to tag a 3sg form —
        // otherwise a missing tag drops the whole verb and renumbers it next refresh.
        // 3sg is regular for almost every verb, so prefer the predicted form when
        // attested; the other slots prefer a genuine irregular over the rule output.
        let third = select_slot(&third_forms, &predicted_third, true);
        let past = select_slot(&past_forms, &predicted_past, false);
        let present_part = select_slot(&pres_part_forms, &predicted_participle, false);
        // Past participle falls back to the chosen past form when unattested.
        let past_part =
            choose_form(&past_part_forms, &predicted_past, false).unwrap_or_else(|| past.clone());

        // The canonical tuple plus one candidate per alternate rank: attested
        // variant conjugations get their own numbered keys. Alternates align by
        // rank per slot (dreamed/dreamt families move together); a slot without a
        // k-th alternate keeps its canonical form. A variant tuple that turns out
        // fully regular is dropped at lemma level by drop_regular, reserving the
        // bare key for the rule engine — so a verb attesting both an irregular and
        // the regular conjugation serves the regular on the bare key and the
        // irregular at suffix 2, exactly like nouns (`die`/`die_2`).
        let acc = by_lemma.entry(lemma).or_default();
        let canonical = vec![
            third.clone(),
            past.clone(),
            present_part.clone(),
            past_part.clone(),
        ];
        acc.observe(canonical.clone(), 0, &entry);
        let alt = [
            slot_alternates(&third_forms, &third),
            slot_alternates(&past_forms, &past),
            slot_alternates(&pres_part_forms, &present_part),
            slot_alternates(&past_part_forms, &past_part),
        ];
        for k in 0..alt.iter().map(|v| v.len()).max().unwrap_or(0) {
            let tuple: Vec<String> = (0..4)
                .map(|i| {
                    alt[i]
                        .get(k)
                        .cloned()
                        .unwrap_or_else(|| canonical[i].clone())
                })
                .collect();
            acc.observe(tuple, (k + 1).min(u8::MAX as usize) as u8, &entry);
        }
    }
    tally.finish("extract_verb_conjugations")?;

    let mut resolved: HashSet<String> = HashSet::new();
    for (lemma, mut acc) in by_lemma {
        let predicted = [
            EnglishCore::verb(
                &lemma,
                &Person::Third,
                &Number::Singular,
                &Tense::Present,
                &Form::Finite,
            ),
            EnglishCore::verb(
                &lemma,
                &Person::Third,
                &Number::Singular,
                &Tense::Past,
                &Form::Finite,
            ),
            EnglishCore::verb(
                &lemma,
                &Person::Third,
                &Number::Singular,
                &Tense::Present,
                &Form::Participle,
            ),
            EnglishCore::verb(
                &lemma,
                &Person::Third,
                &Number::Singular,
                &Tense::Past,
                &Form::Finite,
            ),
        ];
        acc.drop_regular(&predicted, true);
        let (candidates, had_regular) = acc.into_candidates();
        if candidates.is_empty() {
            continue;
        }
        lock.resolve(&lemma, Pos::Verb, candidates, had_regular, date)?;
        resolved.insert(lemma);
    }
    let reap = lock.reap(Pos::Verb, &resolved, date, REAP_MAX_FRACTION);

    println!("Resolved verb conjugations into the lock.");
    Ok(reap)
}

/// One-time reduction of the raw multi-language Wiktextract dump (~20 GB) to
/// English-only lines (~2 GB), cached in `data/intermediate/` so subsequent
/// passes (three extractors + the accuracy harness) each scan a tenth of the
/// data. Lines are copied verbatim — no parsing decisions happen here.
pub fn filter_english_entries(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let input = File::open(input_path)?;
    let reader = BufReader::new(input);
    let mut output = File::create(output_path)?;

    let mut tally = ParseTally::default();
    for line in reader.lines() {
        let line = line?;
        let Some(entry) = tally.parse(&line) else {
            continue;
        };

        if entry.lang_code != "en" {
            continue;
        }

        writeln!(output, "{}", line)?;
    }
    tally.finish("filter_english_entries")?;

    println!("Filtered dataset saved to {}", output_path.display());
    Ok(())
}

/// Strip a `_<digits>` sense suffix from an emitted key (legacy artifact tool).
/// Thin wrapper over `registry::split_key`, the one decoder for the key format.
pub fn strip_trailing_number(word: &str) -> &str {
    split_key(word).map(|(base, _)| base).unwrap_or(word)
}

pub fn analyze_and_write_suffix_rules(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new().from_reader(BufReader::new(file));

    let mut frequency: HashMap<(String, String), usize> = HashMap::new();
    for result in rdr.records() {
        let record = result?;
        let singular_raw = record.get(0).unwrap();
        let plural = record.get(1).unwrap();

        let singular = strip_trailing_number(singular_raw);
        let pair = suffix_rule(singular, plural);
        *frequency.entry(pair).or_insert(0) += 1;
    }

    let mut frequency_rows: Vec<_> = frequency.into_iter().collect();
    frequency_rows.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| a.0.0.cmp(&b.0.0))
            .then_with(|| a.0.1.cmp(&b.0.1))
    });

    let mut writer = WriterBuilder::new().from_path(output_path)?;
    writer.write_record(["singular_suffix", "plural_suffix", "count"])?;

    for ((singular_suffix, plural_suffix), count) in frequency_rows {
        writer.write_record(&[singular_suffix, plural_suffix, count.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract(jsonl: &str, name: &str, f: impl Fn(&Path, &mut Lock)) -> Lock {
        let dir = std::env::temp_dir().join("english_extract_policy_tests");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(name);
        std::fs::write(&path, jsonl).unwrap();
        let mut lock = Lock::new();
        f(&path, &mut lock);
        lock
    }

    fn emitted(lock: &Lock, pos: Pos) -> Vec<(String, Vec<String>)> {
        lock.emittable(pos)
    }

    #[test]
    fn mass_absence_reap_skip_is_reported_to_the_caller() {
        // Pre-populate the lock with many pinned lemmas, then extract from a dump
        // containing almost none of them: the mass-absence guard must refuse to
        // tombstone AND the skip must reach the caller (the pipeline aborts on it
        // unless --allow-partial-refresh) — a truncated dump must never produce a
        // "successful" refresh that leaves stale rows active.
        let dir = std::env::temp_dir().join("english_extract_policy_tests");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("partial.jsonl");
        std::fs::write(
            &path,
            concat!(
                r#"{"word":"worda","pos":"noun","lang_code":"en","forms":[{"form":"wordaen","tags":["plural"]}]}"#,
                "\n"
            ),
        )
        .unwrap();

        let mut lock = Lock::new();
        for c in 'a'..='t' {
            lock.resolve(
                &format!("word{c}"),
                Pos::Noun,
                vec![Candidate::new(vec![format!("word{c}en")])],
                false,
                "d0",
            )
            .unwrap();
        }
        let report = extract_irregular_nouns(&path, &mut lock, "d1").unwrap();
        assert!(
            report.skipped,
            "mass absence must be reported as a skipped reap"
        );
        assert_eq!(report.tombstoned, 0);
        assert!(
            lock.rows()
                .all(|r| r.status == crate::registry::Status::Active),
            "no stale row may be silently tombstoned or altered"
        );
    }

    #[test]
    fn soft_tagged_regular_plural_does_not_reserve_the_bare_key() {
        // sheep: canonical plural "sheep" (plain) + "sheeps" attested only with a
        // soft tag. The soft attestation must NOT reserve the bare key for the rule
        // engine — the irregular takes the bare key ("sheep" -> "sheep", not
        // "sheep2" -> "sheep" with the bare key emitting "sheeps").
        let jsonl = concat!(
            r#"{"word":"sheep","pos":"noun","lang_code":"en","forms":[{"form":"sheep","tags":["plural"]},{"form":"sheeps","tags":["plural","nonstandard"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "sheep.jsonl", |p, l| {
            extract_irregular_nouns(p, l, "d1").unwrap();
        });
        assert_eq!(
            emitted(&lock, Pos::Noun),
            vec![("sheep".to_string(), vec!["sheep".to_string()])]
        );
    }

    #[test]
    fn plain_regular_plural_still_reserves_the_bare_key() {
        // die: both "dies" (regular, plain) and "dice" (irregular, plain) attested.
        // The plain regular attestation reserves the bare key for the rule engine;
        // the irregular is emitted at suffix 2.
        let jsonl = concat!(
            r#"{"word":"die","pos":"noun","lang_code":"en","forms":[{"form":"dies","tags":["plural"]},{"form":"dice","tags":["plural"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "die.jsonl", |p, l| {
            extract_irregular_nouns(p, l, "d1").unwrap();
        });
        assert_eq!(
            emitted(&lock, Pos::Noun),
            vec![("die_2".to_string(), vec!["dice".to_string()])]
        );
    }

    #[test]
    fn formless_adjective_entry_does_not_reserve_the_bare_key() {
        // clever: the gradable sense attests cleverer/cleverest (which the rule
        // engine does NOT predict — two syllables, not -y); a second (uncomparable)
        // sense attests nothing. The formless entry must be skipped, not observed
        // as a synthesized fully-regular candidate — otherwise the bare key emits
        // "more clever" while cleverer/cleverest sit at clever2.
        let jsonl = concat!(
            r#"{"word":"clever","pos":"adj","lang_code":"en","etymology_number":1,"forms":[{"form":"cleverer","tags":["comparative"]},{"form":"cleverest","tags":["superlative"]}]}"#,
            "\n",
            r#"{"word":"clever","pos":"adj","lang_code":"en","etymology_number":2}"#,
            "\n"
        );
        let lock = extract(jsonl, "clever.jsonl", |p, l| {
            extract_irregular_adjectives(p, l, "d1").unwrap();
        });
        assert_eq!(
            emitted(&lock, Pos::Adj),
            vec![(
                "clever".to_string(),
                vec!["cleverer".to_string(), "cleverest".to_string()]
            )]
        );
    }

    #[test]
    fn rule_predicted_adjective_forms_are_not_tabled() {
        // fast: attested faster/fastest now EQUAL the rule prediction, so the
        // candidate is dropped and the rule engine serves the bare key — no table
        // row at all, and no bare-key reservation weirdness.
        let jsonl = concat!(
            r#"{"word":"fast","pos":"adj","lang_code":"en","forms":[{"form":"faster","tags":["comparative"]},{"form":"fastest","tags":["superlative"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "fast.jsonl", |p, l| {
            extract_irregular_adjectives(p, l, "d1").unwrap();
        });
        assert_eq!(emitted(&lock, Pos::Adj), vec![]);
    }

    #[test]
    fn attested_periphrastic_adjective_is_tabled_as_an_exception() {
        // fun: Wiktionary lists "more fun"/"most fun" as the plain forms alongside
        // the informal "funner"/"funnest". The rule engine predicts the suffixal
        // forms (one syllable), so the attested periphrastic must survive as a
        // table row — and the soft-tagged variants are never emitted at all.
        let jsonl = concat!(
            r#"{"word":"fun","pos":"adj","lang_code":"en","forms":[{"form":"more fun","tags":["comparative"]},{"form":"most fun","tags":["superlative"]},{"form":"funner","tags":["comparative","informal"]},{"form":"funnest","tags":["superlative","informal"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "fun.jsonl", |p, l| {
            extract_irregular_adjectives(p, l, "d1").unwrap();
        });
        assert_eq!(
            emitted(&lock, Pos::Adj),
            vec![(
                "fun".to_string(),
                vec!["more fun".to_string(), "most fun".to_string()]
            )]
        );
    }

    #[test]
    fn soft_sense_regular_equal_candidate_does_not_reserve_the_bare_key() {
        // The `bad` pattern: the standard entry attests suppletive worse/worst
        // (plain) with nonstandard badder/"more bad" soft-tagged; a second entry
        // whose senses are ALL slang attests badder/"more bad" plainly. The slang
        // canonical (badder — single-word beats periphrastic) equals the CVC rule
        // prediction, and being all-slang it must NOT reserve the bare key —
        // standard `worse` keeps it (standard sorts before soft at cold start).
        // Because `worse` occupies the bare key, the rule can never serve badder,
        // so the rule-equal soft candidate is KEPT and published as a numbered key
        // rather than dropped; the periphrastic variant follows it.
        let jsonl = concat!(
            r#"{"word":"bad","pos":"adj","lang_code":"en","etymology_number":1,"forms":[{"form":"worse","tags":["comparative"]},{"form":"badder","tags":["comparative","nonstandard"]},{"form":"worst","tags":["superlative"]},{"form":"baddest","tags":["nonstandard","superlative"]}],"senses":[{"glosses":["of low quality"]}]}"#,
            "\n",
            r#"{"word":"bad","pos":"adj","lang_code":"en","etymology_number":1,"forms":[{"form":"badder","tags":["comparative"]},{"form":"more bad","tags":["comparative"]},{"form":"baddest","tags":["superlative"]},{"form":"most bad","tags":["superlative"]}],"senses":[{"tags":["slang"],"glosses":["bold, daring"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "bad.jsonl", |p, l| {
            extract_irregular_adjectives(p, l, "d1").unwrap();
        });
        assert_eq!(
            emitted(&lock, Pos::Adj),
            vec![
                (
                    "bad".to_string(),
                    vec!["worse".to_string(), "worst".to_string()]
                ),
                (
                    "bad_2".to_string(),
                    vec!["badder".to_string(), "baddest".to_string()]
                ),
                (
                    "bad_3".to_string(),
                    vec!["more bad".to_string(), "most bad".to_string()]
                ),
            ]
        );
    }

    #[test]
    fn farther_further_periphrastics_are_admitted() {
        // Locational adjectives inflect with farther/further, listed first in the
        // head template; both attested patterns are published, canonical first.
        let jsonl = concat!(
            r#"{"word":"downhill","pos":"adj","lang_code":"en","forms":[{"form":"farther downhill","tags":["comparative"]},{"form":"further downhill","tags":["comparative"]},{"form":"farthest downhill","tags":["superlative"]},{"form":"furthest downhill","tags":["superlative"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "downhill.jsonl", |p, l| {
            extract_irregular_adjectives(p, l, "d1").unwrap();
        });
        assert_eq!(
            emitted(&lock, Pos::Adj),
            vec![
                (
                    "downhill".to_string(),
                    vec![
                        "farther downhill".to_string(),
                        "farthest downhill".to_string()
                    ]
                ),
                (
                    "downhill_2".to_string(),
                    vec![
                        "further downhill".to_string(),
                        "furthest downhill".to_string()
                    ]
                ),
            ]
        );
    }

    #[test]
    fn sole_soft_rule_equal_candidate_is_truly_redundant_and_dropped() {
        // A slang-only lemma whose only attested pattern equals the rule
        // prediction: no standard sibling to demote, so the bare key is reserved
        // for the rule and no row is emitted — identical output, no dead weight.
        let jsonl = concat!(
            r#"{"word":"zank","pos":"adj","lang_code":"en","forms":[{"form":"zanker","tags":["comparative"]},{"form":"zankest","tags":["superlative"]}],"senses":[{"tags":["slang"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "zank.jsonl", |p, l| {
            extract_irregular_adjectives(p, l, "d1").unwrap();
        });
        assert_eq!(emitted(&lock, Pos::Adj), vec![]);
    }

    #[test]
    fn all_soft_lemma_reserves_bare_and_publishes_variants_at_numbered_keys() {
        // thinky-class: ALL senses soft, attesting the rule-equal suffixal forms
        // AND a periphrastic variant. The rule-equal candidate is dropped and
        // reserves the bare key (the rule serves thinkier there — a bare row
        // would be redundant); the variant is published at the next numbered key.
        let jsonl = concat!(
            r#"{"word":"thinky","pos":"adj","lang_code":"en","forms":[{"form":"thinkier","tags":["comparative"]},{"form":"more thinky","tags":["comparative"]},{"form":"thinkiest","tags":["superlative"]},{"form":"most thinky","tags":["superlative"]}],"senses":[{"tags":["informal"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "thinky.jsonl", |p, l| {
            extract_irregular_adjectives(p, l, "d1").unwrap();
        });
        assert_eq!(
            emitted(&lock, Pos::Adj),
            vec![(
                "thinky_2".to_string(),
                vec!["more thinky".to_string(), "most thinky".to_string()]
            )]
        );
    }

    #[test]
    fn filtered_periphrastic_is_still_published_as_a_numbered_key() {
        // free-like: freer/freest canonical (single-word beats periphrastic for
        // the lowest key), but the plainly-attested periphrastic still gets its
        // own numbered key rather than vanishing.
        let jsonl = concat!(
            r#"{"word":"vree","pos":"adj","lang_code":"en","forms":[{"form":"vreer","tags":["comparative"]},{"form":"more vree","tags":["comparative"]},{"form":"vreest","tags":["superlative"]},{"form":"most vree","tags":["superlative"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "vree.jsonl", |p, l| {
            extract_irregular_adjectives(p, l, "d1").unwrap();
        });
        // vreer/vreest equal the one-syllable rule prediction -> dropped, bare key
        // reserved for the rule; the periphrastic variant is the published key.
        assert_eq!(
            emitted(&lock, Pos::Adj),
            vec![(
                "vree_2".to_string(),
                vec!["more vree".to_string(), "most vree".to_string()]
            )]
        );
    }

    #[test]
    fn verb_variant_conjugations_get_their_own_numbered_keys() {
        // abide-like: three plain pasts/participles attested, "abode" listed FIRST.
        // The canonical tuple takes the lowest key by listing order; the fully
        // regular variant (abided) is dropped and reserves the bare key for the
        // rule engine; the remaining variant (abid) gets the next key.
        let jsonl = concat!(
            r#"{"word":"abide","pos":"verb","lang_code":"en","forms":[{"form":"abides","tags":["third-person","singular","present"]},{"form":"abiding","tags":["participle","present"]},{"form":"abode","tags":["past"]},{"form":"abided","tags":["past"]},{"form":"abid","tags":["past"]},{"form":"abode","tags":["participle","past"]},{"form":"abided","tags":["participle","past"]},{"form":"abid","tags":["participle","past"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "abide.jsonl", |p, l| {
            extract_verb_conjugations(p, l, "d1").unwrap();
        });
        let emit = emitted(&lock, Pos::Verb);
        assert_eq!(emit.len(), 2, "{emit:?}");
        assert_eq!(
            emit[0].0, "abide_2",
            "bare key reserved: the regular conjugation is attested"
        );
        assert_eq!(
            emit[0].1[1], "abode",
            "canonical (first-listed) tuple takes the lowest key"
        );
        assert_eq!(emit[1].0, "abide_3");
        assert_eq!(emit[1].1[1], "abid", "alternate variant gets the next key");
    }

    #[test]
    fn noun_alternate_plurals_get_their_own_numbered_keys() {
        // cactus-like: the regular "cactuses" is plainly attested (dropped; bare
        // key reserved for the rule), and every other plain plural gets its own
        // numbered key in listing order. The soft-tagged one is never emitted.
        let jsonl = concat!(
            r#"{"word":"cactus","pos":"noun","lang_code":"en","forms":[{"form":"cacti","tags":["plural"]},{"form":"cactodes","tags":["plural"]},{"form":"cactuses","tags":["plural"]},{"form":"cactusses","tags":["plural","nonstandard"]}]}"#,
            "\n"
        );
        let lock = extract(jsonl, "cactus.jsonl", |p, l| {
            extract_irregular_nouns(p, l, "d1").unwrap();
        });
        assert_eq!(
            emitted(&lock, Pos::Noun),
            vec![
                ("cactus_2".to_string(), vec!["cacti".to_string()]),
                ("cactus_3".to_string(), vec!["cactodes".to_string()]),
            ],
            "listing order decides numbering; soft form excluded; regular dropped"
        );
    }

    #[test]
    fn formless_verb_entry_does_not_reserve_the_bare_key() {
        // One entry attests an irregular past (3sg omitted -> synthesized); a second
        // entry attests nothing and must be skipped rather than fabricating a
        // fully-regular candidate that would demote the irregular to suffix 2.
        let jsonl = concat!(
            r#"{"word":"zorp","pos":"verb","lang_code":"en","etymology_number":1,"forms":[{"form":"zarp","tags":["past"]},{"form":"zorping","tags":["participle","present"]},{"form":"zarpen","tags":["participle","past"]}]}"#,
            "\n",
            r#"{"word":"zorp","pos":"verb","lang_code":"en","etymology_number":2}"#,
            "\n"
        );
        let lock = extract(jsonl, "zorp.jsonl", |p, l| {
            extract_verb_conjugations(p, l, "d1").unwrap();
        });
        let emit = emitted(&lock, Pos::Verb);
        assert_eq!(emit.len(), 1, "{emit:?}");
        assert_eq!(emit[0].0, "zorp", "irregular takes the bare key");
        assert_eq!(emit[0].1[1], "zarp");
        assert_eq!(
            emit[0].1[0], "zorps",
            "3sg synthesized from the regular rule"
        );
    }
}
