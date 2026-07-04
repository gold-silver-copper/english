//! Stable, append-only assignment of homograph sense numbers.
//!
//! The historic extractor numbered homographs *positionally*: it sorted whatever
//! inflection patterns survived filtering for a lemma and handed out `2, 3, ...`
//! by alphabetical rank. That makes the suffix integer a per-dump artifact — any
//! change to a form, a sense tag, or the regular-rule engine could renumber a
//! lemma and silently swap the meaning of a published key (`run1` <-> `run2`).
//!
//! This module replaces that with a checked-in *assignment lockfile* (one per
//! part of speech). Each emitted key is pinned to a **frozen identity anchor** and
//! a **frozen suffix** (emitted as `lemma` for suffix 1, `lemma_<n>` otherwise —
//! see [`make_key`]). On every refresh, [`Lock::resolve`] matches the dump's
//! candidate senses to existing lock rows by EXACT anchor equality, reuses their
//! pinned suffixes, allocates strictly-higher suffixes for genuinely new senses,
//! and tombstones senses that disappeared (their suffix is retired forever,
//! never reassigned). [`check_immutability`] turns "a published key changed
//! meaning" into a hard, reviewable build failure.
//!
//! # The matching model: exact match or human decision
//!
//! Resolution NEVER guesses. A committed row matches a dump candidate only on
//! exact anchor equality; everything else is tombstone + append, with a
//! *rematch proposal* emitted for a human to review. A sanctioned rematch is an
//! explicit row in `data/assignments/overrides.csv` (reviewed like any other
//! change) — the tool proposes, the human decides, git records why. The two
//! anchor tiers:
//!
//! 1. `sid:en:...` — author-authored senseid (durable by Wiktionary
//!    convention). The ONE in-place drift path: a matching sid asserts
//!    same-sense identity, so the stored forms may update. Drift is only
//!    allowed while the sid is unambiguous — unique in the batch, held by a
//!    single active row, and not `#sig:`-qualified; otherwise the signature
//!    must also match exactly.
//! 2. `sig:<forms>` — content anchor: the identity IS the forms. A `sig:` row
//!    whose forms drift upstream never re-matches; it retires and the new
//!    paradigm appends, so a published key's forms can never silently change
//!    meaning — by construction, not by heuristic.
//!
//! qid/etym remain as informational columns (review aids, coverage stats) and
//! are never POSITIVE matching authority: QIDs are entry-level (shared across
//! all of an entry's senses) and etymology numbers are renumberable section
//! ordinals — both proved capable of silently re-pairing published keys when
//! trusted to select a match. One deliberate exception: a qid (or sid) that
//! conflicts — present on BOTH sides with different values — is a NEGATIVE
//! signal that vetoes an otherwise-exact match (see [`exact_match`]). A veto
//! can only push toward the conservative outcome (tombstone + append +
//! proposal), never select a row, so it cannot re-pair a key; without it, a
//! coincidental form collision could hand a published key to a sense that
//! provably carries a different identity. etym is not a veto (renumbering is
//! routine upstream noise).
//!
//! # How [`Lock::resolve`] proceeds
//!
//! - **Overrides** — sanctioned rematches from `overrides.csv` apply first,
//!   re-anchoring the named row to its reviewed new identity.
//! - **Exact match** — a full row x candidate match matrix ([`exact_match`]),
//!   evaluated globally (no greedy row order): a candidate claimed by rows of
//!   both tiers goes to the durable `sid:` row; a candidate claimed by two rows
//!   of the SAME tier is a hard error (ambiguity never resolves silently).
//! - **Append** — new identities get strictly-higher suffixes, ordered by
//!   [`Candidate::order_key`] (standard-before-soft first, then anchors, then
//!   the dump's listing ordinal, then signature). Anchors stay unique per
//!   (lemma, pos) even against tombstones (`#u<suffix>` disambiguator).
//! - **Tombstone + propose** — unmatched rows retire (suffixes never reused);
//!   when unmatched rows and leftover candidates coexist, each pairing is
//!   emitted as a [`Proposal`] for human review instead of being guessed at.
//!
//! The guiding principle: WHEN IN DOUBT, TOMBSTONE, APPEND, AND ASK. A
//! wrongly-retired key costs an extra suffix and a review; a wrongly-matched
//! key silently changes published behavior, which is the one unrecoverable
//! failure.
//!
//! Cruft is bounded, not eternal: [`Lock::compact`] (via `cargo xtask compact`)
//! drops tombstones and renumbers at a deliberate DATA_MAJOR bump — the
//! versioned, reviewable alternative to letting append-only debris accumulate
//! forever.

use crate::helpers::Pos;
use csv::{ReaderBuilder, WriterBuilder};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

/// CSV header for the lockfiles. Order is load-bearing for the writer/reader.
const LOCK_HEADER: &[&str] = &[
    "lemma",
    "pos",
    "suffix",
    "anchor",
    "qid",
    "sid",
    "etym",
    "status",
    "first_seen",
    "last_seen",
    "forms",
    "glosses",
];

/// Row lifecycle. `Tombstone` is a permanent grave marker, not a deletion:
/// the row stays in the lockfile so its suffix can never be reallocated and
/// its anchor can never be silently re-created with a different meaning
/// ([`check_immutability`] checks both). Tombstones are only ever removed by a
/// sanctioned [`Lock::compact`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Tombstone,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::Tombstone => "tombstone",
        }
    }
}

/// A sense observed in a dump, awaiting a stable key assignment.
#[derive(Debug, Clone, Default)]
pub struct Candidate {
    pub qid: Option<String>,
    pub sid: Option<String>,
    pub etym: Option<u32>,
    /// Emittable inflection columns in canonical order (noun: [plural];
    /// verb: [third, past, present_part, past_part]; adj: [comparative, superlative]).
    pub forms: Vec<String>,
    /// First gloss, stored in the lockfile as a human review aid.
    pub gloss: Option<String>,
    /// True when every contributing sense is soft (slang/nonstandard/...): such a
    /// candidate never reserves the bare key for the rule engine when dropped as
    /// regular-equal. Extraction-time metadata only; not persisted in the lock.
    pub soft_sense: bool,
    /// Cold-start tiebreak among candidates that co-appear with identical anchor
    /// fields (typically several variant patterns from ONE entry): lower ordinals
    /// take lower suffixes. Derived from the dump's listing order (canonical form
    /// first — editorial data), frozen at assignment like everything else, and
    /// never persisted: committed rows keep their suffix by anchor, not ordinal.
    pub ordinal: u8,
}

impl Candidate {
    pub fn new(forms: Vec<String>) -> Self {
        Candidate {
            forms,
            ..Candidate::default()
        }
    }
    fn sig(&self) -> String {
        forms_sig(&self.forms)
    }
    /// A *deterministic, reproducible* total order for assigning suffixes to the
    /// brand-new identities that co-appear at cold start. Standard-before-soft
    /// comes FIRST — a slang-only sense must never outrank a standard sibling
    /// for a low key, regardless of what metadata either happens to carry —
    /// then the anchor fields (qid, then sid, then etym), then the listing
    /// `ordinal` (several variant patterns from one entry number
    /// canonical-first, per the head template's editorial order), and only then
    /// the form signature. Note this is purely a tiebreak among co-appearing new
    /// senses (the result is frozen immediately after) — NOT a claim about which
    /// sense is "primary": a missing qid sorts as the empty string, so a
    /// metadata-less sense can sort ahead of a metadata-bearing one. The
    /// bare-vs-`2` choice among genuine homographs is intentionally
    /// arbitrary-but-stable.
    fn order_key(&self) -> (bool, String, String, u32, u8, String) {
        (
            self.soft_sense,
            self.qid.clone().unwrap_or_default(),
            self.sid.clone().unwrap_or_default(),
            self.etym.unwrap_or(0),
            self.ordinal,
            self.sig(),
        )
    }
}

/// One pinned key in a lockfile.
#[derive(Debug, Clone)]
pub struct LockRow {
    pub lemma: String,
    pub pos: Pos,
    pub suffix: u32,
    /// Frozen primary identity, chosen at creation. The immutability guard keys on
    /// this; it changes only through a reviewed override.
    pub anchor: String,
    /// Informational metadata (review aids / coverage stats), refreshed from the
    /// last matched candidate. Never POSITIVE matching authority; a conflicting
    /// qid/sid on both sides is a negative veto in [`exact_match`] (it can only
    /// force tombstone + append, never select a match).
    pub qid: Option<String>,
    pub sid: Option<String>,
    pub etym: Option<u32>,
    pub status: Status,
    pub first_seen: String,
    pub last_seen: String,
    /// Current emittable forms (the values written into the PHF tables).
    pub forms: Vec<String>,
    pub gloss: Option<String>,
}

impl LockRow {
    fn sig(&self) -> String {
        forms_sig(&self.forms)
    }

    /// Fold a freshly-matched candidate into this row: refresh the forms, the
    /// informational metadata, and `last_seen`. The frozen `anchor` and `suffix`
    /// are deliberately untouched. qid/sid/etym are OVERWRITTEN (not filled-if-
    /// missing): they carry no matching authority, and keeping them current
    /// prevents a stale value from misleading a reviewer.
    fn apply_match(&mut self, c: &Candidate, date: &str) {
        self.qid = c.qid.clone();
        self.sid = c.sid.clone();
        self.etym = c.etym;
        if c.gloss.is_some() {
            self.gloss = c.gloss.clone();
        }
        self.forms = c.forms.clone();
        self.last_seen = date.to_string();
        self.status = Status::Active;
    }
}

/// The output of resolving one lemma: a stable emitted key (already through
/// [`make_key`]) plus the forms to publish under it. Purely informational for
/// callers — the authoritative state lives in the mutated [`Lock`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub key: String,
    pub forms: Vec<String>,
}

/// A sanctioned rematch: "the row at (lemma, pos, suffix), currently anchored on
/// `old_anchor`, is the same sense as the dump candidate identified by
/// `new_anchor` — re-anchor it and carry the key forward." Authored by a human in
/// `data/assignments/overrides.csv` after reviewing a [`Proposal`]; this is the
/// ONLY way a `sig:`-anchored row's forms can change without retiring the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Override {
    pub lemma: String,
    pub pos: Pos,
    pub suffix: u32,
    pub old_anchor: String,
    pub new_anchor: String,
}

/// The reviewed overrides file. An override is *pending* while its row still
/// carries `old_anchor` AND is active (the next refresh applies it), *applied*
/// once the row carries `new_anchor` (inert; safe to delete), and *stale* if
/// the row carries neither, does not exist, or is tombstoned while still
/// pending (a hard check failure — the file must never drift out of
/// correspondence with the locks).
///
/// Workflow: overrides apply to ACTIVE rows only, so sanctioning a proposal
/// means re-running the refresh from the committed baseline — `git restore
/// data/assignments`, add the override, `cargo xtask refresh-data` — NOT
/// stacking the override on top of the already-tombstoned refresh output.
#[derive(Debug, Clone, Default)]
pub struct Overrides {
    pub rows: Vec<Override>,
}

const OVERRIDES_HEADER: &[&str] = &["lemma", "pos", "suffix", "old_anchor", "new_anchor"];

impl Overrides {
    /// Load `overrides.csv`; a missing file is an empty set, a malformed row is a
    /// hard error (same strictness as [`Lock::load`]).
    pub fn load(path: impl AsRef<Path>) -> Result<Overrides, Box<dyn Error>> {
        let path = path.as_ref();
        let mut out = Overrides::default();
        if !path.exists() {
            return Ok(out);
        }
        let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
        for record in rdr.records() {
            let record = record?;
            let pos = Pos::parse(record.get(1).unwrap_or("").trim())
                .ok_or_else(|| format!("bad pos in override row: {record:?}"))?;
            let suffix: u32 = record
                .get(2)
                .unwrap_or("")
                .trim()
                .parse()
                .map_err(|_| format!("invalid suffix in override row: {record:?}"))?;
            let old_anchor = record.get(3).unwrap_or("").trim().to_string();
            let new_anchor = record.get(4).unwrap_or("").trim().to_string();
            if old_anchor.is_empty() || new_anchor.is_empty() || old_anchor == new_anchor {
                return Err(format!("invalid anchors in override row: {record:?}").into());
            }
            out.rows.push(Override {
                lemma: record.get(0).unwrap_or("").trim().to_string(),
                pos,
                suffix,
                old_anchor,
                new_anchor,
            });
        }
        Ok(out)
    }

    /// Cross-check every override against the lock it targets. `require_applied`
    /// is set right after a refresh (the dump was just consulted, so a still-
    /// pending override means its target candidate was absent — stale by
    /// definition); check-registry runs with it unset (pending is a legitimate
    /// waiting state between committing an override and running the refresh).
    pub fn check(&self, lock: &Lock, pos: Pos, require_applied: bool) -> Vec<String> {
        let mut v = Vec::new();
        for o in self.rows.iter().filter(|o| o.pos == pos) {
            let row = lock
                .map
                .get(&(o.lemma.clone(), o.pos.as_str().to_string()))
                .and_then(|rows| rows.iter().find(|r| r.suffix == o.suffix));
            match row {
                None => v.push(format!(
                    "override {} [{}] suffix {}: no such lock row — stale override, remove it",
                    o.lemma,
                    o.pos.as_str(),
                    o.suffix
                )),
                Some(r) if r.anchor == o.new_anchor && r.status == Status::Active => {} // applied; inert
                Some(r) if r.anchor == o.new_anchor => v.push(format!(
                    "override {} [{}] suffix {}: row carries the applied anchor `{}` but is \
                     tombstoned, so the sanctioned rematch is not actually published — remove \
                     or regenerate the override from the active baseline",
                    o.lemma,
                    o.pos.as_str(),
                    o.suffix,
                    o.new_anchor
                )),
                Some(r) if r.anchor == o.old_anchor && r.status == Status::Tombstone => {
                    // Application only ever targets ACTIVE rows, so a pending
                    // override whose row is already tombstoned (the refresh that
                    // emitted the proposal was committed as-is) can never apply —
                    // accepting it as "pending" would just fail the NEXT refresh.
                    v.push(format!(
                        "override {} [{}] suffix {}: the target row is tombstoned, so this \
                         override can never apply. Restore the lockfiles to the committed \
                         baseline (git restore data/assignments) and re-run refresh-data so it \
                         applies to the active row, or remove the override.",
                        o.lemma,
                        o.pos.as_str(),
                        o.suffix
                    ));
                }
                Some(r) if r.anchor == o.old_anchor => {
                    if require_applied {
                        v.push(format!(
                            "override {} [{}] suffix {}: still pending after the refresh — the \
                             candidate `{}` was not in the dump (stale override?)",
                            o.lemma,
                            o.pos.as_str(),
                            o.suffix,
                            o.new_anchor
                        ));
                    }
                }
                Some(r) => v.push(format!(
                    "override {} [{}] suffix {}: row anchor `{}` matches neither old `{}` nor \
                     new `{}` — stale override, remove or fix it",
                    o.lemma,
                    o.pos.as_str(),
                    o.suffix,
                    r.anchor,
                    o.old_anchor,
                    o.new_anchor
                )),
            }
        }
        v.sort();
        v
    }

    /// Whether this set sanctions replacing `old_anchor` at (lemma, pos) with a
    /// row holding `new_anchor` at the same suffix. Used by [`check_immutability`]
    /// to waive exactly the anchor transition the override describes.
    fn waives(&self, lemma: &str, pos: &str, suffix: u32, old_anchor: &str) -> Option<&Override> {
        self.rows.iter().find(|o| {
            o.lemma == lemma
                && o.pos.as_str() == pos
                && o.suffix == suffix
                && o.old_anchor == old_anchor
        })
    }
}

/// A rematch the resolver REFUSED to make (exact matching found no claim, but an
/// unmatched pinned row and a leftover candidate coexist for the lemma). Written
/// to `pending-rematches.csv` for human review; approving one means adding the
/// corresponding row to `overrides.csv`, restoring the lockfiles to the
/// committed baseline, and re-running the refresh (see [`Overrides`]) — the
/// override then carries the key forward instead of the tombstone + append.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    pub lemma: String,
    pub pos: Pos,
    pub suffix: u32,
    pub old_anchor: String,
    pub old_forms: Vec<String>,
    /// The anchor an override would name to accept this pairing.
    pub candidate_anchor: String,
    pub candidate_forms: Vec<String>,
    pub candidate_gloss: Option<String>,
}

/// An entire lockfile (all rows for one part of speech), indexed by `(lemma, pos)`.
#[derive(Debug, Default, Clone)]
pub struct Lock {
    map: BTreeMap<(String, String), Vec<LockRow>>,
    /// Human-facing notes accumulated during resolution (override applications,
    /// tombstones, ambiguous cases) — surfaced to the maintainer, never affects
    /// determinism.
    pub notes: Vec<String>,
    /// Refused rematches accumulated during resolution, for `pending-rematches.csv`.
    pub proposals: Vec<Proposal>,
    /// Sanctioned rematches, set before resolution (see [`Lock::set_overrides`]).
    overrides: Vec<Override>,
}

impl Lock {
    pub fn new() -> Self {
        Lock::default()
    }

    /// Insert a fully-formed row (bootstrap/test path — normal refreshes only
    /// create rows through [`Lock::resolve`]). Keeps the bucket suffix-sorted,
    /// which [`Lock::save`] and Phase A's suffix-order iteration rely on.
    pub fn insert_row(&mut self, row: LockRow) {
        let key = (row.lemma.clone(), row.pos.as_str().to_string());
        let bucket = self.map.entry(key).or_default();
        bucket.push(row);
        bucket.sort_by_key(|r| r.suffix);
    }

    pub fn rows(&self) -> impl Iterator<Item = &LockRow> {
        self.map.values().flat_map(|v| v.iter())
    }

    /// All active rows for a part of speech, as `(key, forms)`, sorted by key —
    /// exactly what the PHF generators consume.
    pub fn emittable(&self, pos: Pos) -> Vec<(String, Vec<String>)> {
        let mut out: Vec<(String, Vec<String>)> = self
            .map
            .values()
            .flat_map(|v| v.iter())
            .filter(|r| r.pos == pos && r.status == Status::Active)
            .map(|r| (make_key(&r.lemma, r.suffix), r.forms.clone()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Active-row emit-key collisions for `pos`: any `make_key` produced by more than
    /// one active row (e.g. suffix 0 and suffix 1 both map to the bare lemma). Each
    /// entry names the offending key and the colliding anchors. Empty = safe to emit;
    /// a non-empty result would otherwise surface only as an opaque `phf_map!`
    /// duplicate-key *compile* error in the downstream crate.
    pub fn emit_collisions(&self, pos: Pos) -> Vec<String> {
        let mut by_key: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for r in self.map.values().flat_map(|v| v.iter()) {
            if r.pos == pos && r.status == Status::Active {
                by_key
                    .entry(make_key(&r.lemma, r.suffix))
                    .or_default()
                    .push(&r.anchor);
            }
        }
        by_key
            .into_iter()
            .filter(|(_, anchors)| anchors.len() > 1)
            .map(|(key, anchors)| {
                format!(
                    "emit key `{key}` [{}] is produced by {} active rows (anchors: {})",
                    pos.as_str(),
                    anchors.len(),
                    anchors.join(", ")
                )
            })
            .collect()
    }

    /// Strict internal-consistency checks over the lock's own contents (no external
    /// baseline). Catches the kinds of corruption a hand-edit could introduce that
    /// [`check_immutability`] does not: a suffix below 1 (suffix 0 would alias the
    /// bare key), an unknown anchor tier, an anchor whose frozen payload no longer
    /// names the row's own identity (sid field / form signature),
    /// an active row whose forms column has the wrong arity or an empty column
    /// (which would emit `""` inflections into the PHF tables via the
    /// pad-with-default generation path), and any emit-key collision. Returns
    /// human-readable violations; empty = valid.
    pub fn validate(&self) -> Vec<String> {
        let mut v = Vec::new();
        let mut suffix_occupants: BTreeMap<(String, String, u32), Vec<String>> = BTreeMap::new();
        for r in self.rows() {
            suffix_occupants
                .entry((r.lemma.clone(), r.pos.as_str().to_string(), r.suffix))
                .or_default()
                .push(r.anchor.clone());
            // Tombstones are exempt: they are never emitted, and their forms are
            // whatever they last were.
            if r.status == Status::Active {
                let expected = match r.pos {
                    Pos::Noun => 1,
                    Pos::Verb => 4,
                    Pos::Adj => 2,
                };
                if r.forms.len() != expected {
                    v.push(format!(
                        "{} [{}] suffix {} has {} form column(s), expected {} — regeneration would pad with empty strings",
                        r.lemma,
                        r.pos.as_str(),
                        r.suffix,
                        r.forms.len(),
                        expected
                    ));
                } else if r.forms.iter().any(|f| f.is_empty()) {
                    v.push(format!(
                        "{} [{}] suffix {} has an empty form column — it would emit a \"\" inflection",
                        r.lemma,
                        r.pos.as_str(),
                        r.suffix
                    ));
                }
            }
            if r.suffix < 1 {
                v.push(format!(
                    "{} [{}] has suffix {} (must be >= 1)",
                    r.lemma,
                    r.pos.as_str(),
                    r.suffix
                ));
            }
            // The frozen anchor payload must NAME the row it froze: `exact_match`
            // and the immutability guard trust the anchor as the row's identity,
            // so a payload that disagrees with the row's own columns is a lie a
            // DATA_MAJOR-waived change could otherwise smuggle in. Tombstones are
            // exempt from payload checks (like forms arity): they never emit or
            // rematch, and check_immutability guards them by anchor string alone.
            match Anchor::parse(&r.anchor) {
                None => v.push(format!(
                    "{} [{}] anchor `{}` has unknown tier `{}` (exact matching knows only sid/sig; \
                     run `cargo xtask migrate-anchors` for a pre-migration lock)",
                    r.lemma,
                    r.pos.as_str(),
                    r.anchor,
                    r.anchor.split(':').next().unwrap_or("")
                )),
                Some(anchor) if r.status == Status::Active => {
                    // The sid: tier names a durable senseid; that senseid must be
                    // present in and agree with the sid column.
                    if let Some(sid) = anchor.senseid() {
                        match r.sid.as_deref() {
                            None => v.push(format!(
                                "{} [{}] anchor `{}` is sid-tier but the sid field is empty",
                                r.lemma,
                                r.pos.as_str(),
                                r.anchor
                            )),
                            Some(field) if field != sid => v.push(format!(
                                "{} [{}] anchor `{}` names sid `{sid}` but the sid field is `{field}`",
                                r.lemma,
                                r.pos.as_str(),
                                r.anchor
                            )),
                            _ => {}
                        }
                    }
                    // Any frozen form signature (a `sig:` payload or a `#sig:`
                    // qualifier) must equal the row's current forms; a bare `sid:`
                    // freezes none, so its forms are free to drift.
                    if let Some(frozen) = anchor.frozen_sig()
                        && frozen != r.sig()
                    {
                        v.push(format!(
                            "{} [{}] anchor `{}` freezes sig `{frozen}` but the forms are `{}`",
                            r.lemma,
                            r.pos.as_str(),
                            r.anchor,
                            r.sig()
                        ));
                    }
                }
                // Tombstones are exempt from payload checks: they never emit or
                // rematch, and check_immutability guards them by anchor string alone.
                Some(_) => {}
            }
        }
        v.extend(duplicate_anchor_violations(self.rows()));
        for ((lemma, pos, suffix), anchors) in &suffix_occupants {
            if anchors.len() > 1 {
                v.push(format!(
                    "{lemma} [{pos}] suffix {suffix} is held by {} identities at once: {}",
                    anchors.len(),
                    anchors.join(", ")
                ));
            }
        }
        for pos in [Pos::Adj, Pos::Noun, Pos::Verb] {
            v.extend(self.emit_collisions(pos));
        }
        v.sort();
        v
    }

    /// Install the reviewed overrides (normally the full contents of
    /// `overrides.csv`; rows for other parts of speech are ignored at use).
    pub fn set_overrides(&mut self, overrides: &Overrides) {
        self.overrides = overrides.rows.clone();
    }

    /// Resolve one lemma's candidates against the committed lock, mutating the lock
    /// in place and returning the stable assignments. `had_regular` records whether
    /// a regular-prediction-equal sense was seen for this lemma (and dropped by the
    /// caller as a runtime fall-through), which decides cold-start numbering.
    ///
    /// Errors are AMBIGUITY, not I/O: a candidate exactly claimed by two rows of
    /// the same tier, or a row exactly claimed by two candidates. Exact matching
    /// makes these structurally near-impossible (signatures are batch-unique, sid
    /// drift is gated on unambiguity), so an error means the lock and dump
    /// genuinely contradict each other — the refresh must stop and a human must
    /// resolve it (usually via `overrides.csv`), never pick a winner by order.
    pub fn resolve(
        &mut self,
        lemma: &str,
        pos: Pos,
        candidates: Vec<Candidate>,
        had_regular: bool,
        date: &str,
    ) -> Result<Vec<Assignment>, String> {
        let mut notes = Vec::new();
        let mut proposals = Vec::new();
        let key = (lemma.to_string(), pos.as_str().to_string());
        let overrides: Vec<Override> = self.for_lemma_overrides(lemma, pos);

        let assignments = {
            let rows = self.map.entry(key).or_default();
            let original_len = rows.len();
            let mut matched = vec![false; original_len];
            let mut remaining = candidates;
            let mut assignments: Vec<Assignment> = Vec::new();

            // Committed-bucket identity facts, snapshotted BEFORE overrides mutate
            // any anchor. sid drift-allowance must be a property of the committed
            // lock, not of the post-override matched set: an override consuming one
            // twin must NOT loosen the content gate on its sibling. `sid_dup` (a
            // senseid carried by >1 candidate this batch) and `sid_shared` (a
            // senseid held by >1 active row in the committed bucket) each force
            // content agreement for a sid match; both feed the override guard below
            // and the exact-match matrix.
            let sid_dup = duplicate_sids(&remaining);
            let sid_shared: std::collections::HashSet<String> = {
                let mut counts: BTreeMap<String, u32> = BTreeMap::new();
                for r in rows.iter().filter(|r| r.status == Status::Active) {
                    if let Some(sid) =
                        Anchor::parse(&r.anchor).and_then(|a| a.senseid().map(str::to_string))
                    {
                        *counts.entry(sid).or_default() += 1;
                    }
                }
                counts
                    .into_iter()
                    .filter(|(_, n)| *n > 1)
                    .map(|(s, _)| s)
                    .collect()
            };

            // -- Overrides: sanctioned rematches apply before anything else --
            // A pending override re-anchors its row to the reviewed new identity and
            // consumes the candidate that identity names. Consumption is content-
            // aware and refuses to guess: a `new_anchor` that names several sibling
            // candidates (a bare `sid:` with variant forms), or a candidate that some
            // OTHER active row exactly claims by content, is left UNAPPLIED — the
            // override stays pending and Overrides::check() turns that into a hard
            // error, rather than silently picking a variant or robbing a sibling.
            for ov in &overrides {
                let Some(ri) = (0..original_len)
                    .find(|&ri| rows[ri].suffix == ov.suffix && rows[ri].status == Status::Active)
                else {
                    continue;
                };
                if rows[ri].anchor == ov.new_anchor {
                    notes.push(format!(
                        "override for {lemma}{} already applied — remove it from overrides.csv",
                        suffix_note(ov.suffix)
                    ));
                    continue;
                }
                if rows[ri].anchor != ov.old_anchor {
                    continue;
                }
                let Some(new_anchor) = Anchor::parse(&ov.new_anchor) else {
                    continue; // unparseable new_anchor -> stays pending, flagged by check()
                };
                let matching: Vec<usize> = remaining
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| new_anchor.matches_candidate(c))
                    .map(|(i, _)| i)
                    .collect();
                if matching.len() > 1 {
                    notes.push(format!(
                        "override for {lemma}{} NOT applied: new anchor `{}` names {} sibling \
                         candidates by senseid alone — add a `#sig:` qualifier to disambiguate",
                        suffix_note(ov.suffix),
                        ov.new_anchor,
                        matching.len()
                    ));
                    continue;
                }
                let Some(&ci) = matching.first() else {
                    continue; // candidate absent -> stays pending
                };
                // Never rob a sibling: if another active row exactly content-claims
                // this candidate, applying would silently tombstone that row. Refuse
                // and leave the override pending so the conflict is reviewed.
                let robs_sibling = (0..original_len).any(|rj| {
                    rj != ri
                        && !matched[rj]
                        && rows[rj].status == Status::Active
                        && exact_match(&rows[rj], &remaining[ci], &sid_dup, &sid_shared)
                });
                if robs_sibling {
                    notes.push(format!(
                        "override for {lemma}{} NOT applied: the sanctioned candidate is also an \
                         exact content match for another active row — resolve the conflict",
                        suffix_note(ov.suffix)
                    ));
                    continue;
                }
                let c = remaining.remove(ci);
                notes.push(format!(
                    "override applied: {lemma}{} re-anchored `{}` -> `{}` ({} -> {})",
                    suffix_note(rows[ri].suffix),
                    ov.old_anchor,
                    ov.new_anchor,
                    rows[ri].sig(),
                    c.sig()
                ));
                rows[ri].anchor = ov.new_anchor.clone();
                rows[ri].apply_match(&c, date);
                assignments.push(Assignment {
                    key: make_key(lemma, rows[ri].suffix),
                    forms: rows[ri].forms.clone(),
                });
                matched[ri] = true;
            }

            // -- Exact match: full row x candidate matrix, no greedy order --

            // For each candidate, collect every row that exactly claims it.
            let mut claims: Vec<(usize, usize)> = Vec::new(); // (candidate idx, row idx)
            for (ci, c) in remaining.iter().enumerate() {
                let claimants: Vec<usize> = (0..original_len)
                    .filter(|&ri| {
                        rows[ri].status == Status::Active
                            && !matched[ri]
                            && exact_match(&rows[ri], c, &sid_dup, &sid_shared)
                    })
                    .collect();
                let (sid_rows, sig_rows): (Vec<usize>, Vec<usize>) = claimants
                    .into_iter()
                    .partition(|&ri| Anchor::parse(&rows[ri].anchor).is_some_and(|a| a.is_sid()));
                if sid_rows.len() > 1 || sig_rows.len() > 1 {
                    let anchors: Vec<&str> = sid_rows
                        .iter()
                        .chain(sig_rows.iter())
                        .map(|&ri| rows[ri].anchor.as_str())
                        .collect();
                    return Err(format!(
                        "AMBIGUOUS: {lemma} [{}] candidate `{}` is exactly claimed by {} rows of \
                         the same tier ({}); refusing to pick one — resolve via overrides.csv",
                        pos.as_str(),
                        c.sig(),
                        sid_rows.len().max(sig_rows.len()),
                        anchors.join(", ")
                    ));
                }
                // A candidate claimed by both tiers goes to the durable sid row;
                // the sig row is left to tombstone (its content now belongs to a
                // sense with an explicit durable identity).
                if let Some(&ri) = sid_rows.first().or(sig_rows.first()) {
                    claims.push((ci, ri));
                }
            }
            // Two candidates claiming one row would mean two batch-distinct
            // signatures both exactly matched it — structurally impossible, but
            // ambiguity must never resolve silently, so keep the guard.
            {
                let mut per_row: BTreeMap<usize, u32> = BTreeMap::new();
                for &(_, ri) in &claims {
                    *per_row.entry(ri).or_default() += 1;
                }
                if let Some((&ri, _)) = per_row.iter().find(|(_, n)| **n > 1) {
                    return Err(format!(
                        "AMBIGUOUS: {lemma} [{}] row `{}` is exactly claimed by multiple dump \
                         candidates; refusing to pick one — resolve via overrides.csv",
                        pos.as_str(),
                        rows[ri].anchor
                    ));
                }
            }
            // Apply the 1:1 matches (descending candidate index so removal is stable).
            claims.sort_by_key(|&(ci, _)| std::cmp::Reverse(ci));
            for (ci, ri) in claims {
                let c = remaining.remove(ci);
                rows[ri].apply_match(&c, date);
                assignments.push(Assignment {
                    key: make_key(lemma, rows[ri].suffix),
                    forms: c.forms,
                });
                matched[ri] = true;
            }

            // -- Propose, never guess: unmatched pinned rows x leftover candidates --
            let unmatched_active: Vec<usize> = (0..original_len)
                .filter(|&ri| !matched[ri] && rows[ri].status == Status::Active)
                .collect();
            if !unmatched_active.is_empty() && !remaining.is_empty() {
                notes.push(format!(
                    "AMBIGUOUS: {lemma} [{}] has {} unmatched pinned sense(s) and {} unmatched dump sense(s) \
                     with no exact anchor match; tombstoning the pinned one(s) and allocating new suffix(es). \
                     Rematch proposals recorded — review pending-rematches and sanction any real match via overrides.csv.",
                    pos.as_str(),
                    unmatched_active.len(),
                    remaining.len()
                ));
                for &ri in &unmatched_active {
                    for c in &remaining {
                        proposals.push(Proposal {
                            lemma: lemma.to_string(),
                            pos,
                            suffix: rows[ri].suffix,
                            old_anchor: rows[ri].anchor.clone(),
                            old_forms: rows[ri].forms.clone(),
                            candidate_anchor: Anchor::for_candidate(c, &sid_dup).to_string(),
                            candidate_forms: c.forms.clone(),
                            candidate_gloss: c.gloss.clone(),
                        });
                    }
                }
            }

            // -- Append: allocate strictly-higher suffixes for new identities --
            let base = if original_len == 0 {
                if had_regular { 2 } else { 1 }
            } else {
                rows.iter().map(|r| r.suffix).max().unwrap_or(1) + 1
            };
            // Anchors must stay unique within a (lemma, pos), INCLUDING against
            // tombstones: a sense that vanished and later reappears re-derives the
            // same anchor, which would collide with its own retired row. Disambiguate
            // the new row with its (unique, append-only) suffix so the immutability
            // guard can still tell the two apart.
            let mut taken: std::collections::HashSet<String> =
                rows.iter().map(|r| r.anchor.clone()).collect();
            remaining.sort_by_key(|c| c.order_key());
            for (offset, c) in remaining.into_iter().enumerate() {
                let suffix = base + offset as u32;
                let base_anchor = Anchor::for_candidate(&c, &sid_dup);
                let anchor = base_anchor.to_string();
                let anchor = if taken.contains(&anchor) {
                    base_anchor.with_uniq(suffix).to_string()
                } else {
                    anchor
                };
                taken.insert(anchor.clone());
                let mut row = LockRow {
                    lemma: lemma.to_string(),
                    pos,
                    suffix,
                    anchor,
                    qid: None,
                    sid: None,
                    etym: None,
                    status: Status::Active,
                    first_seen: date.to_string(),
                    last_seen: date.to_string(),
                    forms: Vec::new(),
                    gloss: None,
                };
                row.apply_match(&c, date);
                assignments.push(Assignment {
                    key: make_key(lemma, suffix),
                    forms: row.forms.clone(),
                });
                rows.push(row);
            }

            // -- Tombstone pinned senses that vanished from the dump --
            for ri in 0..original_len {
                if !matched[ri] && rows[ri].status == Status::Active {
                    rows[ri].status = Status::Tombstone;
                    rows[ri].last_seen = date.to_string();
                    notes.push(format!(
                        "tombstoned {lemma}{} (anchor {}); suffix retired, never reused",
                        suffix_note(rows[ri].suffix),
                        rows[ri].anchor
                    ));
                }
            }

            rows.sort_by_key(|r| r.suffix);
            assignments
        };

        self.notes.append(&mut notes);
        self.proposals.append(&mut proposals);
        Ok(assignments)
    }

    fn for_lemma_overrides(&self, lemma: &str, pos: Pos) -> Vec<Override> {
        self.overrides
            .iter()
            .filter(|o| o.lemma == lemma && o.pos == pos)
            .cloned()
            .collect()
    }

    /// One-time migration to the exact-match anchor model: every row whose frozen
    /// anchor is a qid/etym tier (including composites and `#u` variants) is
    /// re-anchored to its current form signature; `sid:` and `sig:` rows are kept
    /// as they are. Suffixes, forms, and statuses are untouched — the emitted
    /// tables cannot change. Returns the number of rows re-anchored. This rewrites
    /// frozen anchors, so it must ship with a DATA_MAJOR bump (`cargo xtask
    /// migrate-anchors` does both).
    pub fn migrate_to_exact_anchors(&mut self) -> usize {
        let mut migrated = 0;
        for rows in self.map.values_mut() {
            let mut taken: std::collections::HashSet<String> = rows
                .iter()
                .filter(|r| Anchor::parse(&r.anchor).is_some())
                .map(|r| r.anchor.clone())
                .collect();
            // Suffix order keeps `#u` disambiguation deterministic.
            let mut order: Vec<usize> = (0..rows.len()).collect();
            order.sort_by_key(|&i| rows[i].suffix);
            for i in order {
                let r = &mut rows[i];
                if Anchor::parse(&r.anchor).is_some() {
                    continue; // already sid/sig tier
                }
                let base_anchor = Anchor::Sig {
                    forms_sig: forms_sig(&r.forms),
                    uniq: None,
                };
                let anchor = base_anchor.to_string();
                let anchor = if taken.contains(&anchor) {
                    base_anchor.with_uniq(r.suffix).to_string()
                } else {
                    anchor
                };
                taken.insert(anchor.clone());
                r.anchor = anchor;
                migrated += 1;
            }
        }
        migrated
    }

    /// Compact the lock for a **data-major release**: delete every tombstone and
    /// close the suffix gaps they (and ambiguous-drift appends) left behind, so the
    /// published keys become dense again (`x_2`,`x_5`,`x_7` -> `x_2`,`x_3`,`x_4`).
    ///
    /// This RENUMBERS published keys — the one operation the lock otherwise exists
    /// to prevent — so it must only ever land together with a `DATA_MAJOR` bump
    /// (`cargo xtask compact` does both; `check-registry` only waives cross-version
    /// immutability for a PR that bumps the major).
    ///
    /// Two invariants survive compaction:
    /// - relative suffix order is preserved (a lower-numbered sense stays lower);
    /// - the bare slot (suffix 1) is never *newly* occupied: whether it was reserved
    ///   for the rule engine at cold start (`had_regular`) is not recorded in the
    ///   lock, so promoting a row into it would be a guess that could shadow a
    ///   regular inflection.
    pub fn compact(&mut self) -> CompactReport {
        let mut report = CompactReport::default();
        for rows in self.map.values_mut() {
            let before = rows.len();
            rows.retain(|r| r.status == Status::Active);
            report.tombstones_dropped += before - rows.len();
            rows.sort_by_key(|r| r.suffix);
            let mut next = 2;
            for r in rows.iter_mut() {
                let new = if r.suffix == 1 {
                    1
                } else {
                    let n = next;
                    next += 1;
                    n
                };
                if new != r.suffix {
                    report.renumbered += 1;
                }
                r.suffix = new;
            }
        }
        self.map.retain(|_, rows| !rows.is_empty());
        report
    }

    /// Load a lockfile; a missing file is an EMPTY lock (first run), while a
    /// malformed row is a hard error — silent coercion once turned a corrupted
    /// suffix into 0, which aliases the bare key. Columns are positional
    /// (see `LOCK_HEADER`); appending columns is backward-compatible,
    /// reordering is not.
    pub fn load(path: impl AsRef<Path>) -> Result<Lock, Box<dyn Error>> {
        let path = path.as_ref();
        let mut lock = Lock::new();
        if !path.exists() {
            return Ok(lock);
        }
        let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
        for record in rdr.records() {
            let record = record?;
            let pos = Pos::parse(record.get(1).unwrap_or("").trim())
                .ok_or_else(|| format!("bad pos in lock row: {record:?}"))?;
            // Strict parsing: a corrupted suffix or status is a hard error, not a
            // silent coercion (a suffix that fell back to 0 used to alias the bare key).
            let suffix: u32 = record
                .get(2)
                .unwrap_or("")
                .trim()
                .parse()
                .map_err(|_| format!("invalid suffix in lock row: {record:?}"))?;
            let status = match record.get(7).unwrap_or("active").trim() {
                "active" => Status::Active,
                "tombstone" => Status::Tombstone,
                other => {
                    return Err(format!("invalid status {other:?} in lock row: {record:?}").into());
                }
            };
            let forms = split_forms(record.get(10).unwrap_or(""));
            let row = LockRow {
                lemma: record.get(0).unwrap_or("").to_string(),
                pos,
                suffix,
                anchor: record.get(3).unwrap_or("").to_string(),
                qid: opt(record.get(4)),
                sid: opt(record.get(5)),
                // Same strictness as suffix/status: an EMPTY cell is a legitimate
                // None, but non-empty garbage is a hard error — a silently-dropped
                // etym once looked identical to a never-recorded one.
                etym: match record.get(6).map(str::trim) {
                    None | Some("") => None,
                    Some(s) => Some(
                        s.parse::<u32>()
                            .map_err(|_| format!("invalid etym in lock row: {record:?}"))?,
                    ),
                },
                status,
                first_seen: record.get(8).unwrap_or("").to_string(),
                last_seen: record.get(9).unwrap_or("").to_string(),
                forms,
                gloss: opt(record.get(11)),
            };
            lock.insert_row(row);
        }
        Ok(lock)
    }

    /// Write the lockfile deterministically: (lemma, pos) buckets in BTreeMap
    /// order, rows within a bucket by suffix. Byte-stable output matters —
    /// lock diffs are the human review surface for every refresh, and
    /// `check-registry` parses committed lockfiles straight out of git.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let mut wtr = WriterBuilder::new().from_path(path)?;
        wtr.write_record(LOCK_HEADER)?;
        // Deterministic order: (lemma, pos, suffix). BTreeMap already orders the
        // (lemma, pos) keys; rows within a bucket are kept sorted by suffix.
        for bucket in self.map.values() {
            for r in bucket {
                wtr.write_record(&[
                    r.lemma.clone(),
                    r.pos.as_str().to_string(),
                    r.suffix.to_string(),
                    r.anchor.clone(),
                    r.qid.clone().unwrap_or_default(),
                    r.sid.clone().unwrap_or_default(),
                    r.etym.map(|e| e.to_string()).unwrap_or_default(),
                    r.status.as_str().to_string(),
                    r.first_seen.clone(),
                    r.last_seen.clone(),
                    r.forms.join("|"),
                    r.gloss.clone().unwrap_or_default(),
                ])?;
            }
        }
        wtr.flush()?;
        Ok(())
    }

    /// Coverage histogram by FROZEN anchor tier — answers "how much of the lock
    /// can survive form drift without a review?" Only `sid:` rows can (the
    /// durable tier); `sig:` rows retire on drift and need an override to
    /// rematch. Buckets come from the anchor prefix, not the informational
    /// qid/etym columns, which are no longer matching authority.
    pub fn coverage(&self) -> Coverage {
        let mut c = Coverage::default();
        for r in self.rows() {
            if r.status != Status::Active {
                c.tombstoned += 1;
                continue;
            }
            c.active += 1;
            match Anchor::parse(&r.anchor) {
                Some(Anchor::Sid { .. }) => c.by_sid += 1,
                Some(Anchor::Sig { .. }) => c.by_sig += 1,
                None => c.legacy += 1,
            }
        }
        c
    }

    /// Tombstone active rows for `pos` whose lemma was NOT resolved this refresh —
    /// i.e. the lemma vanished from the dump entirely, or all its senses became
    /// regular (so [`resolve`] was never called for it and its Phase-D tombstoning
    /// never ran). Without this, a fully-absent lemma keeps emitting a now-stale
    /// irregular form forever and [`check_immutability`] sees nothing changed.
    ///
    /// Guarded against a partial/corrupt dump: if reaping would retire more than
    /// `max_fraction` of this pos's active rows, nothing is tombstoned and the
    /// situation is reported, so a truncated dump cannot silently gut the lockfile.
    pub fn reap(
        &mut self,
        pos: Pos,
        resolved: &std::collections::HashSet<String>,
        date: &str,
        max_fraction: f64,
    ) -> ReapReport {
        let pos_str = pos.as_str();
        let mut active = 0usize;
        let mut target_keys: Vec<(String, String)> = Vec::new();
        for ((lemma, p), rows) in self.map.iter() {
            if p != pos_str {
                continue;
            }
            let n_active = rows.iter().filter(|r| r.status == Status::Active).count();
            active += n_active;
            if n_active > 0 && !resolved.contains(lemma) {
                target_keys.push((lemma.clone(), p.clone()));
            }
        }

        let target_rows: usize = target_keys
            .iter()
            .filter_map(|k| self.map.get(k))
            .flat_map(|rows| rows.iter())
            .filter(|r| r.status == Status::Active)
            .count();

        if active > 0 && (target_rows as f64) > (active as f64 * max_fraction) {
            self.notes.push(format!(
                "REAP SKIPPED [{pos_str}]: {target_rows}/{active} active rows ({:.0}%) are absent from this \
                 dump — refusing to tombstone (likely a partial/bad dump); no rows changed.",
                100.0 * target_rows as f64 / active as f64
            ));
            return ReapReport {
                tombstoned: 0,
                skipped: true,
            };
        }

        let mut notes = Vec::new();
        let mut tombstoned = 0;
        for key in &target_keys {
            if let Some(rows) = self.map.get_mut(key) {
                for r in rows.iter_mut() {
                    if r.status == Status::Active {
                        r.status = Status::Tombstone;
                        r.last_seen = date.to_string();
                        tombstoned += 1;
                        notes.push(format!(
                            "reaped {}{} (anchor {}); lemma absent from dump, suffix retired",
                            r.lemma,
                            suffix_note(r.suffix),
                            r.anchor
                        ));
                    }
                }
            }
        }
        self.notes.append(&mut notes);
        ReapReport {
            tombstoned,
            skipped: false,
        }
    }
}

/// Summary of a [`Lock::reap`] pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReapReport {
    pub tombstoned: usize,
    pub skipped: bool,
}

/// Summary of a [`Lock::compact`] pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CompactReport {
    pub tombstones_dropped: usize,
    pub renumbered: usize,
}

impl CompactReport {
    pub fn is_noop(&self) -> bool {
        self.tombstones_dropped == 0 && self.renumbered == 0
    }
}

#[derive(Debug, Default)]
pub struct Coverage {
    pub active: usize,
    pub tombstoned: usize,
    /// Rows anchored on a durable senseid (survive form drift in place).
    pub by_sid: usize,
    /// Rows anchored on their form signature (retire on drift; override to rematch).
    pub by_sig: usize,
    /// Pre-migration qid/etym tiers — should be zero after `migrate-anchors`.
    pub legacy: usize,
}

/// Compare a previous (committed) lock against a freshly-resolved one and return a
/// list of *violations*: ways in which an already-published key changed meaning.
/// Empty = safe to ship.
///
/// `overrides` waives EXACTLY the transitions it describes: an old row whose
/// anchor was replaced at the same suffix by the override's `new_anchor` is a
/// sanctioned rematch, not a drop.
///
/// Beyond anchors and suffixes, FORMS are immutable too: a row that keeps its
/// anchor must keep its forms, except for bare `sid:` rows (senseid identity is
/// the one sanctioned in-place drift path; a `#sig:`-qualified sid anchor bakes
/// the forms into the identity, so those may not drift either). This closes the
/// hand-edit hole: without it, editing a sig-anchored row's forms in place would
/// republish a key with different meaning and no anchor/suffix change to flag.
pub fn check_immutability(old: &Lock, new: &Lock, overrides: &Overrides) -> Vec<String> {
    let mut violations = Vec::new();

    // Index new rows by frozen identity and by (lemma,pos,suffix).
    let mut new_by_anchor: BTreeMap<(String, String, String), &LockRow> = BTreeMap::new();
    let mut new_by_suffix: BTreeMap<(String, String, u32), Vec<&LockRow>> = BTreeMap::new();
    for r in new.rows() {
        let akey = (
            r.lemma.clone(),
            r.pos.as_str().to_string(),
            r.anchor.clone(),
        );
        new_by_anchor.insert(akey, r);
        new_by_suffix
            .entry((r.lemma.clone(), r.pos.as_str().to_string(), r.suffix))
            .or_default()
            .push(r);
    }

    // Every frozen anchor must identify exactly one row within a (lemma,pos);
    // a duplicate means two keys share an identity (and the guard can't track them).
    violations.extend(duplicate_anchor_violations(new.rows()));

    for r in old.rows() {
        let id = (
            r.lemma.clone(),
            r.pos.as_str().to_string(),
            r.anchor.clone(),
        );
        match new_by_anchor.get(&id) {
            None => {
                // A sanctioned override rematch: the same suffix now carries the
                // override's new anchor. Anything else is a drop.
                let waived = overrides
                    .waives(&r.lemma, r.pos.as_str(), r.suffix, &r.anchor)
                    .is_some_and(|ov| {
                        new_by_suffix
                            .get(&(r.lemma.clone(), r.pos.as_str().to_string(), r.suffix))
                            .is_some_and(|occ| {
                                occ.iter().any(|nr| {
                                    nr.anchor == ov.new_anchor && nr.status == Status::Active
                                })
                            })
                    });
                if !waived {
                    // BOTH statuses are append-only: dropping an active row loses a
                    // published key, and dropping a tombstone un-retires its suffix
                    // (appends allocate from max(suffix)+1, so a deleted top
                    // tombstone lets the NEXT refresh re-mint the retired key for a
                    // brand-new sense — across two individually-green PRs).
                    let why = match r.status {
                        Status::Active => "an active key must be tombstoned, not deleted",
                        Status::Tombstone => {
                            "tombstones are removed only by a sanctioned compact (DATA_MAJOR bump)"
                        }
                    };
                    violations.push(format!(
                        "{} [{}] anchor `{}` (suffix {}, {}) was DROPPED; {why}",
                        r.lemma,
                        r.pos.as_str(),
                        r.anchor,
                        r.suffix,
                        r.status.as_str()
                    ));
                }
            }
            Some(nr) => {
                if nr.suffix != r.suffix {
                    violations.push(format!(
                        "{} [{}] anchor `{}` MOVED suffix {} -> {} (would change a published key's meaning)",
                        r.lemma, r.pos.as_str(), r.anchor, r.suffix, nr.suffix
                    ));
                }
                if r.status == Status::Tombstone && nr.status == Status::Active {
                    violations.push(format!(
                        "{} [{}] anchor `{}` was REVIVED from tombstone (retired identities must stay retired)",
                        r.lemma, r.pos.as_str(), r.anchor
                    ));
                }
                let sid_drift_allowed =
                    Anchor::parse(&r.anchor).is_some_and(|a| a.allows_form_drift());
                if r.forms != nr.forms && !sid_drift_allowed {
                    violations.push(format!(
                        "{} [{}] anchor `{}` CHANGED FORMS in place ({} -> {}); only bare sid: \
                         rows may drift — a sig-anchored change of meaning needs an override",
                        r.lemma,
                        r.pos.as_str(),
                        r.anchor,
                        forms_sig(&r.forms),
                        forms_sig(&nr.forms)
                    ));
                }
            }
        }

        // A retired suffix must never be occupied by a *different* identity.
        if r.status == Status::Tombstone {
            let occupants =
                new_by_suffix.get(&(r.lemma.clone(), r.pos.as_str().to_string(), r.suffix));
            if let Some(occ) = occupants {
                for o in occ {
                    if o.anchor != r.anchor && o.status == Status::Active {
                        violations.push(format!(
                            "{} [{}] suffix {} (retired with anchor `{}`) was REUSED by anchor `{}`",
                            r.lemma, r.pos.as_str(), r.suffix, r.anchor, o.anchor
                        ));
                    }
                }
            }
        }
    }

    // Within the new lock, each (lemma,pos,suffix) must be held by exactly one identity.
    for ((lemma, pos, suffix), occ) in &new_by_suffix {
        if occ.len() > 1 {
            violations.push(format!(
                "{lemma} [{pos}] suffix {suffix} is held by {} identities at once: {}",
                occ.len(),
                occ.iter()
                    .map(|r| r.anchor.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    violations.sort();
    violations
}

// ----- free helpers -----

/// Rows sharing a (lemma, pos, anchor) triple mean two keys claim one frozen
/// identity — the immutability guard can't tell them apart. One violation per
/// over-shared anchor. Shared by [`Lock::validate`] and [`check_immutability`],
/// which each own a different context (internal-only vs cross-version).
fn duplicate_anchor_violations<'a>(rows: impl Iterator<Item = &'a LockRow>) -> Vec<String> {
    let mut anchor_counts: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    for r in rows {
        *anchor_counts
            .entry((
                r.lemma.clone(),
                r.pos.as_str().to_string(),
                r.anchor.clone(),
            ))
            .or_default() += 1;
    }
    anchor_counts
        .iter()
        .filter(|(_, n)| **n > 1)
        .map(|((lemma, pos, anchor), n)| {
            format!(
                "{lemma} [{pos}] anchor `{anchor}` is shared by {n} rows (anchors must be unique per lemma)"
            )
        })
        .collect()
}

/// Suffix 1 emits the bare lemma; >=2 appends `_<n>`. The underscore separator is
/// unambiguous: extraction filters out any lemma containing `_` (word_is_proper),
/// so a trailing `_<digits>` can ONLY be a sense suffix — digit-bearing words
/// (`mp3`, `F16`, `F2`, `co2`) can never collide with a numbered key.
pub fn make_key(lemma: &str, suffix: u32) -> String {
    if suffix <= 1 {
        lemma.to_string()
    } else {
        format!("{lemma}_{suffix}")
    }
}

/// Inverse of [`make_key`]: `Some((lemma, suffix))` when `key` carries a
/// `_<digits>` sense suffix, `None` for a bare key. This is the ONE decoder for
/// the emitted-key format inside the extractor — bootstrap seeding, the accuracy
/// harness, and the artifact tools all go through it, so the format can never
/// drift apart between them. A digit run that overflows `u32` is a hard error
/// (panic), never a silent coercion: a coerced suffix would alias the bare key,
/// and such a key can only come from a corrupt generated table.
pub fn split_key(key: &str) -> Option<(&str, u32)> {
    let (base, digits) = key.rsplit_once('_')?;
    if base.is_empty() || digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let suffix = digits
        .parse()
        .unwrap_or_else(|_| panic!("sense suffix overflows u32 in key {key:?}"));
    Some((base, suffix))
}

/// The canonical form-signature encoding (`forms.join("|")`) and its `sig:`
/// anchor spelling. [`Candidate::sig`], [`LockRow::sig`], anchor construction,
/// and bootstrap seeding all funnel through these two functions — the frozen
/// `sig:` anchors in committed lockfiles must keep corresponding to freshly
/// computed signatures across refreshes, so the encoding must never fork.
pub fn forms_sig(forms: &[String]) -> String {
    forms.join("|")
}

/// See [`forms_sig`]. The bootstrap-tier frozen identity for a row with no
/// dump anchors.
pub fn sig_anchor(forms: &[String]) -> String {
    format!("sig:{}", forms_sig(forms))
}

/// Both sides carry a value and the values differ — identity evidence that the
/// row and candidate are DIFFERENT senses. `None` on either side is no opinion.
fn strong_conflict<T: PartialEq>(a: &Option<T>, b: &Option<T>) -> bool {
    matches!((a, b), (Some(x), Some(y)) if x != y)
}

fn suffix_note(suffix: u32) -> String {
    if suffix <= 1 {
        String::new()
    } else {
        format!("_{suffix}")
    }
}

fn opt(s: Option<&str>) -> Option<String> {
    match s {
        Some(v) if !v.trim().is_empty() => Some(v.to_string()),
        _ => None,
    }
}

fn split_forms(s: &str) -> Vec<String> {
    if s.is_empty() {
        Vec::new()
    } else {
        s.split('|').map(|x| x.to_string()).collect()
    }
}

/// Senseids carried by more than one candidate in the batch (a Wiktionary entry
/// can list several inflection variants of ONE sense — they share its sid). A
/// duplicated sid cannot identify a row by itself; content must agree too.
fn duplicate_sids(candidates: &[Candidate]) -> std::collections::HashSet<String> {
    let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
    for c in candidates {
        if let Some(s) = &c.sid {
            *counts.entry(s).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(s, _)| s.to_string())
        .collect()
}

/// A parsed frozen anchor — the ONE typed model for the `sid:`/`sig:`/`#sig:`/`#u`
/// grammar. The lockfile stores the *string* form (`LockRow.anchor`) as the
/// byte-stable source of truth; every semantic operation funnels through
/// [`Anchor::parse`]/`Anchor::to_string` so the parsing can never drift apart
/// between call sites — it used to be re-derived ad hoc in five places with
/// subtly different `#` handling. Legacy qid/etym tiers deliberately do NOT parse
/// (`parse` returns `None`); `validate` reports them and `migrate-anchors`
/// rewrites them straight off the raw string.
///
/// Grammar (exactly the shapes the resolver emits), any of which may carry a
/// trailing `#u<n>` reappearance disambiguator:
///   `sig:<forms>`                content identity (identity IS the forms)
///   `sid:<senseid>`              durable identity (forms may drift in place)
///   `sid:<senseid>#sig:<forms>`  durable identity, content-qualified
#[derive(Debug, Clone, PartialEq, Eq)]
enum Anchor {
    Sig {
        forms_sig: String,
        uniq: Option<u32>,
    },
    Sid {
        senseid: String,
        forms_sig: Option<String>,
        uniq: Option<u32>,
    },
}

impl Anchor {
    /// Parse a stored anchor string; `None` for a legacy/unknown tier.
    ///
    /// Only the reserved `#sig:` and trailing `#u<n>` sequences are delimiters — a
    /// `#` anywhere else (e.g. inside a senseid like `en:totality#English`) stays
    /// part of the senseid and round-trips intact, closing the truncation hole the
    /// old first-`#` split had. Ingest also rejects `#`-bearing senseids (see
    /// `Entry::lowest_senseid`), so the pathological `#sig:`/`#u<n>`-suffixed
    /// senseid never reaches an anchor.
    fn parse(s: &str) -> Option<Anchor> {
        let (body, uniq) = split_uniq(s);
        if let Some(rest) = body.strip_prefix("sig:") {
            return Some(Anchor::Sig {
                forms_sig: rest.to_string(),
                uniq,
            });
        }
        if let Some(rest) = body.strip_prefix("sid:") {
            let (senseid, forms_sig) = match rest.split_once("#sig:") {
                Some((id, f)) => (id.to_string(), Some(f.to_string())),
                None => (rest.to_string(), None),
            };
            return Some(Anchor::Sid {
                senseid,
                forms_sig,
                uniq,
            });
        }
        None
    }

    /// The durable senseid this anchor names, if any (`sid:` tier).
    fn senseid(&self) -> Option<&str> {
        match self {
            Anchor::Sid { senseid, .. } => Some(senseid),
            Anchor::Sig { .. } => None,
        }
    }

    /// The form signature this anchor freezes, if any: the whole payload of a
    /// `sig:` anchor, the `#sig:` qualifier of a content-qualified `sid:`, or
    /// `None` for a bare `sid:` (whose forms are free to drift).
    fn frozen_sig(&self) -> Option<&str> {
        match self {
            Anchor::Sig { forms_sig, .. } => Some(forms_sig),
            Anchor::Sid { forms_sig, .. } => forms_sig.as_deref(),
        }
    }

    /// Whether this anchor's own structure lets the row's forms drift in place:
    /// only a bare `sid:` (durable identity, no content qualifier). A `sig:`
    /// anchor IS its forms; a `#sig:`-qualified `sid:` bakes the forms in.
    fn allows_form_drift(&self) -> bool {
        matches!(
            self,
            Anchor::Sid {
                forms_sig: None,
                ..
            }
        )
    }

    fn is_sid(&self) -> bool {
        matches!(self, Anchor::Sid { .. })
    }

    /// Attach a `#u<suffix>` reappearance disambiguator.
    fn with_uniq(mut self, n: u32) -> Anchor {
        match &mut self {
            Anchor::Sig { uniq, .. } | Anchor::Sid { uniq, .. } => *uniq = Some(n),
        }
        self
    }

    /// The frozen primary identity for a NEW row: the senseid when one exists
    /// (content-qualified when the batch shares it), else the form signature.
    fn for_candidate(c: &Candidate, sid_dup: &std::collections::HashSet<String>) -> Anchor {
        match &c.sid {
            Some(s) if !sid_dup.contains(s) => Anchor::Sid {
                senseid: s.clone(),
                forms_sig: None,
                uniq: None,
            },
            Some(s) => Anchor::Sid {
                senseid: s.clone(),
                forms_sig: Some(c.sig()),
                uniq: None,
            },
            None => Anchor::Sig {
                forms_sig: c.sig(),
                uniq: None,
            },
        }
    }

    /// Whether a candidate is the sense this anchor names (used to apply an
    /// override's `new_anchor`): the senseid must match, and any frozen form
    /// signature must equal the candidate's. A bare `sid:` names a sense by
    /// senseid ALONE, so it can match several form-variant siblings — the
    /// override path guards that ambiguity explicitly (see [`Lock::resolve`]).
    fn matches_candidate(&self, c: &Candidate) -> bool {
        match self {
            Anchor::Sig { forms_sig, .. } => c.sig() == *forms_sig,
            Anchor::Sid {
                senseid, forms_sig, ..
            } => {
                c.sid.as_deref() == Some(senseid.as_str())
                    && forms_sig.as_ref().is_none_or(|f| c.sig() == *f)
            }
        }
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Anchor::Sig { forms_sig, uniq } => {
                write!(out, "sig:{forms_sig}")?;
                if let Some(n) = uniq {
                    write!(out, "#u{n}")?;
                }
            }
            Anchor::Sid {
                senseid,
                forms_sig,
                uniq,
            } => {
                write!(out, "sid:{senseid}")?;
                if let Some(f) = forms_sig {
                    write!(out, "#sig:{f}")?;
                }
                if let Some(n) = uniq {
                    write!(out, "#u{n}")?;
                }
            }
        }
        Ok(())
    }
}

/// Peel a trailing `#u<digits>` reappearance disambiguator, returning the body
/// and the parsed suffix. Form signatures and senseids never contain `#` in
/// practice (forms fail `word_is_proper`; ingest rejects `#`-senseids), so this
/// is unambiguous.
fn split_uniq(s: &str) -> (&str, Option<u32>) {
    if let Some((head, tail)) = s.rsplit_once("#u")
        && let Ok(n) = tail.parse::<u32>()
    {
        (head, Some(n))
    } else {
        (s, None)
    }
}

/// The exact-match predicate — the ONLY way a committed row pairs with a dump
/// candidate outside a reviewed override.
///
/// - `sig:` rows match on exact form-signature equality: the identity IS the
///   forms, so a match can never change what the key emits.
/// - `sid:` rows match on exact senseid equality. The stored forms may then
///   drift in place (the senseid asserts same-sense identity), but ONLY while
///   that reading is unambiguous: the sid must be unique in the batch, held by
///   a single active row, and not `#sig:`-qualified — otherwise the signature
///   must agree too, which reduces sid matching to content matching exactly
///   where siblings would be indistinguishable.
///
/// A conflicting durable senseid or QID on either side vetoes any match: a
/// coincidental form collision must never hand this row's published key to a
/// sense that carries a different identity (a swap [`check_immutability`] could
/// only see through the forms guard, after the damage is committed).
fn exact_match(
    row: &LockRow,
    c: &Candidate,
    sid_dup: &std::collections::HashSet<String>,
    sid_shared: &std::collections::HashSet<String>,
) -> bool {
    if strong_conflict(&row.qid, &c.qid) || strong_conflict(&row.sid, &c.sid) {
        return false;
    }
    match Anchor::parse(&row.anchor) {
        Some(Anchor::Sid {
            senseid, forms_sig, ..
        }) => {
            let Some(csid) = &c.sid else { return false };
            if csid != &senseid {
                return false;
            }
            let sig_required =
                forms_sig.is_some() || sid_dup.contains(&senseid) || sid_shared.contains(&senseid);
            !sig_required || row.sig() == c.sig()
        }
        Some(Anchor::Sig { .. }) => row.sig() == c.sig(),
        // Legacy qid/etym tiers never match; `Lock::validate` rejects them until
        // `migrate-anchors` has re-anchored the lock.
        None => false,
    }
}

/// Write refused rematches for human review. One row per (retired row,
/// leftover candidate) pairing; approving one means copying its first five
/// columns into `overrides.csv` (they are already in that file's shape),
/// restoring the lockfiles to the committed baseline, and re-running the
/// refresh so the override applies to the still-active row.
pub fn save_proposals(
    proposals: &[Proposal],
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().from_path(path)?;
    wtr.write_record(OVERRIDES_HEADER.iter().copied().chain([
        "old_forms",
        "candidate_forms",
        "candidate_gloss",
    ]))?;
    for p in proposals {
        wtr.write_record(&[
            p.lemma.clone(),
            p.pos.as_str().to_string(),
            p.suffix.to_string(),
            p.old_anchor.clone(),
            p.candidate_anchor.clone(),
            p.old_forms.join("|"),
            p.candidate_forms.join("|"),
            p.candidate_gloss.clone().unwrap_or_default(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(forms: &[&str]) -> Candidate {
        Candidate::new(forms.iter().map(|s| s.to_string()).collect())
    }

    fn cand_e(forms: &[&str], etym: u32) -> Candidate {
        let mut c = cand(forms);
        c.etym = Some(etym);
        c
    }

    fn cand_q(forms: &[&str], qid: &str) -> Candidate {
        let mut c = cand(forms);
        c.qid = Some(qid.to_string());
        c
    }

    fn cand_s(forms: &[&str], sid: &str) -> Candidate {
        let mut c = cand(forms);
        c.sid = Some(sid.to_string());
        c
    }

    fn keys(a: &[Assignment]) -> Vec<String> {
        let mut k: Vec<String> = a.iter().map(|x| x.key.clone()).collect();
        k.sort();
        k
    }

    fn no_ov() -> Overrides {
        Overrides::default()
    }

    #[test]
    fn cold_start_reproduces_legacy_lie() {
        // No regular-equal sense dropped -> start at suffix 1; alphabetical-by-forms order.
        let mut lock = Lock::new();
        let recline = cand(&["lies", "lay", "lying", "lain"]);
        let untruth = cand(&["lies", "lied", "lying", "lied"]);
        let a = lock
            .resolve("lie", Pos::Verb, vec![untruth, recline], false, "d1")
            .unwrap();
        let mut by_key: BTreeMap<_, _> =
            a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key.remove("lie").unwrap()[1], "lay"); // recline is the bare key
        assert_eq!(by_key.remove("lie_2").unwrap()[1], "lied"); // untruth is lie_2
    }

    #[test]
    fn cold_start_with_regular_starts_at_two() {
        // die: regular plural dropped (had_regular), single irregular -> die_2, no bare die.
        let mut lock = Lock::new();
        let a = lock
            .resolve("die", Pos::Noun, vec![cand(&["dice"])], true, "d1")
            .unwrap();
        assert_eq!(keys(&a), vec!["die_2".to_string()]);
    }

    #[test]
    fn reordered_forms_do_not_swap_keys() {
        // Establish lie/lie_2, then a later dump returns the senses in a different
        // order. Keys must hold (sig anchors are order-independent).
        let mut lock = Lock::new();
        lock.resolve(
            "lie",
            Pos::Verb,
            vec![
                cand(&["lies", "lay", "lying", "lain"]),
                cand(&["lies", "lied", "lying", "lied"]),
            ],
            false,
            "d1",
        )
        .unwrap();
        // Round-trip through CSV to ensure persistence carries identity.
        let dir = std::env::temp_dir().join("english_registry_test_swap");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("verb.lock.csv");
        lock.save(&path).unwrap();
        let mut lock2 = Lock::load(&path).unwrap();

        let a = lock2
            .resolve(
                "lie",
                Pos::Verb,
                vec![
                    cand(&["lies", "lied", "lying", "lied"]),
                    cand(&["lies", "lay", "lying", "lain"]),
                ],
                false,
                "d2",
            )
            .unwrap();
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["lie"][1], "lay", "recline must stay the bare key");
        assert_eq!(by_key["lie_2"][1], "lied", "untruth must stay lie_2");
    }

    #[test]
    fn sid_anchor_holds_and_drifts_in_place() {
        // A senseid is the ONE identity that carries a key through form drift.
        let mut lock = Lock::new();
        lock.resolve(
            "lie",
            Pos::Verb,
            vec![
                cand_s(&["lies", "lay", "lying", "lain"], "en:to rest"),
                cand_s(&["lies", "lied", "lying", "lied"], "en:say falsehoods"),
            ],
            false,
            "d1",
        )
        .unwrap();
        let dir = std::env::temp_dir().join("english_registry_test_siddrift");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("verb.lock.csv");
        lock.save(&path).unwrap();
        let before = Lock::load(&path).unwrap();

        // d2: recline's past participle edited lain -> layed; sid unchanged.
        let a = lock
            .resolve(
                "lie",
                Pos::Verb,
                vec![
                    cand_s(&["lies", "lied", "lying", "lied"], "en:say falsehoods"),
                    cand_s(&["lies", "lay", "lying", "layed"], "en:to rest"),
                ],
                false,
                "d2",
            )
            .unwrap();
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        // sid sort order puts "en:say falsehoods" first at cold start (as in the
        // real verb lock); recline drifts in place at lie_2.
        assert_eq!(by_key["lie"][1], "lied");
        assert_eq!(
            by_key["lie_2"][3], "layed",
            "sid row kept its key, forms updated"
        );
        // Bare-sid drift is the sanctioned exception to forms immutability.
        assert!(check_immutability(&before, &lock, &no_ov()).is_empty());
    }

    #[test]
    fn new_sense_appends_and_existing_holds() {
        let mut lock = Lock::new();
        lock.resolve(
            "run",
            Pos::Verb,
            vec![cand(&["runs", "ran", "running", "run"])],
            false,
            "d1",
        )
        .unwrap();
        let a = lock
            .resolve(
                "run",
                Pos::Verb,
                vec![
                    cand(&["runs", "ran", "running", "run"]),
                    cand(&["runs", "runned", "running", "runned"]),
                ],
                false,
                "d2",
            )
            .unwrap();
        assert_eq!(keys(&a), vec!["run".to_string(), "run_2".to_string()]);
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["run"][1], "ran", "original sense keeps the bare key");
        assert_eq!(by_key["run_2"][1], "runned", "new sense appended at run_2");
    }

    #[test]
    fn vanished_sense_is_tombstoned_and_suffix_never_reused() {
        let mut lock = Lock::new();
        lock.resolve(
            "x",
            Pos::Noun,
            vec![cand(&["xen"]), cand(&["xes"])],
            false,
            "d1",
        )
        .unwrap();
        let a = lock
            .resolve(
                "x",
                Pos::Noun,
                vec![cand(&["xes"]), cand(&["xim"])],
                false,
                "d2",
            )
            .unwrap();
        // xen's suffix (1) is retired; xes keeps suffix 2; new sense gets 3 (not 1).
        assert_eq!(keys(&a), vec!["x_2".to_string(), "x_3".to_string()]);
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["x_2"][0], "xes");
        assert_eq!(by_key["x_3"][0], "xim");
    }

    #[test]
    fn resolve_is_idempotent() {
        let mut lock = Lock::new();
        let mk = || {
            vec![
                cand(&["lies", "lay", "lying", "lain"]),
                cand(&["lies", "lied", "lying", "lied"]),
            ]
        };
        let a1 = lock.resolve("lie", Pos::Verb, mk(), false, "d1").unwrap();
        let a2 = lock.resolve("lie", Pos::Verb, mk(), false, "d1").unwrap();
        assert_eq!(keys(&a1), keys(&a2));
        assert_eq!(keys(&a1), vec!["lie".to_string(), "lie_2".to_string()]);
    }

    #[test]
    fn immutability_guard_flags_a_swap() {
        let mut old = Lock::new();
        old.resolve(
            "lie",
            Pos::Verb,
            vec![
                cand(&["lies", "lay", "lying", "lain"]),
                cand(&["lies", "lied", "lying", "lied"]),
            ],
            false,
            "d1",
        )
        .unwrap();
        // Hand-craft a "new" lock where the two anchors swapped suffixes.
        let mut new = Lock::new();
        for r in old.rows() {
            let mut nr = r.clone();
            nr.suffix = if r.suffix == 1 { 2 } else { 1 };
            new.insert_row(nr);
        }
        let v = check_immutability(&old, &new, &no_ov());
        assert!(!v.is_empty(), "swap must be flagged");
        assert!(v.iter().any(|m| m.contains("MOVED suffix")));
    }

    #[test]
    fn immutability_guard_allows_append() {
        let mut old = Lock::new();
        old.resolve("die", Pos::Noun, vec![cand(&["dice"])], true, "d1")
            .unwrap();
        let mut new = Lock::new();
        new.resolve("die", Pos::Noun, vec![cand(&["dice"])], true, "d1")
            .unwrap();
        new.resolve(
            "die",
            Pos::Noun,
            vec![cand(&["dice"]), cand(&["dies_alt"])],
            true,
            "d2",
        )
        .unwrap();
        assert!(check_immutability(&old, &new, &no_ov()).is_empty());
    }

    #[test]
    fn forms_drift_in_place_is_flagged_for_sig_rows() {
        // A hand edit that changes a sig-anchored row's forms without touching
        // anchor or suffix is EXACTLY the invisible meaning change the guard
        // exists for; a bare sid row may drift (senseid identity).
        let mut old = Lock::new();
        old.insert_row(row("w", 1, "sig:foo", None, &["foo"]));
        old.insert_row(row_s("v", 1, "sid:en:x", "en:x", &["va"]));
        let mut new = Lock::new();
        new.insert_row(row("w", 1, "sig:foo", None, &["bar"])); // edited in place
        new.insert_row(row_s("v", 1, "sid:en:x", "en:x", &["vb"])); // sid drift: fine
        let v = check_immutability(&old, &new, &no_ov());
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("CHANGED FORMS"), "{v:?}");
        assert!(v[0].contains("`sig:foo`"), "{v:?}");
    }

    #[test]
    fn single_sense_drift_tombstones_and_proposes() {
        // Exact-or-ask: even a 1-out/1-in form drift is never re-matched
        // automatically — the old key retires, the new paradigm appends, and a
        // rematch proposal records the pairing for a human to sanction.
        let mut lock = Lock::new();
        lock.resolve("w", Pos::Noun, vec![cand(&["foo"])], false, "d1")
            .unwrap();
        let a = lock
            .resolve("w", Pos::Noun, vec![cand(&["bar"])], false, "d2")
            .unwrap();
        assert_eq!(keys(&a), vec!["w_2".to_string()]);
        let old = lock.rows().find(|r| r.suffix == 1).unwrap();
        assert_eq!(old.status, Status::Tombstone);
        assert_eq!(
            old.forms,
            vec!["foo".to_string()],
            "pinned forms never overwritten"
        );
        assert_eq!(lock.proposals.len(), 1);
        let p = &lock.proposals[0];
        assert_eq!((p.suffix, p.old_anchor.as_str()), (1, "sig:foo"));
        assert_eq!(p.candidate_anchor, "sig:bar");
        assert!(lock.notes.iter().any(|n| n.contains("AMBIGUOUS")));
    }

    #[test]
    fn conflicting_qid_vetoes_content_match() {
        // Same forms, different durable identity: the impostor must not inherit
        // the key even though the content matches exactly.
        let mut lock = Lock::new();
        lock.resolve("w", Pos::Noun, vec![cand_q(&["foo"], "Q1")], false, "d1")
            .unwrap();
        let a = lock
            .resolve("w", Pos::Noun, vec![cand_q(&["foo"], "Q2")], false, "d2")
            .unwrap();
        assert_eq!(keys(&a), vec!["w_2".to_string()]);
        let old = lock.rows().find(|r| r.suffix == 1).unwrap();
        assert_eq!(old.status, Status::Tombstone);
        // The reappearing content re-derives the same sig anchor; it must be
        // disambiguated against the tombstone, not collide with it.
        let anchors: Vec<String> = lock.rows().map(|r| r.anchor.clone()).collect();
        let distinct: std::collections::HashSet<_> = anchors.iter().collect();
        assert_eq!(anchors.len(), distinct.len(), "{anchors:?}");
    }

    #[test]
    fn durable_sid_outranks_content_claim() {
        // A candidate exactly claimed by a sig row (content) AND a sid row
        // (durable identity) goes to the sid row; the sig row retires.
        let mut lock = Lock::new();
        lock.resolve(
            "w",
            Pos::Noun,
            vec![cand(&["foo"]), cand_s(&["bar"], "en:s")],
            false,
            "d1",
        )
        .unwrap(); // sig:foo @1, sid:en:s @2
        let a = lock
            .resolve("w", Pos::Noun, vec![cand_s(&["foo"], "en:s")], false, "d2")
            .unwrap();
        assert_eq!(keys(&a), vec!["w_2".to_string()]);
        let sid_row = lock.rows().find(|r| r.suffix == 2).unwrap();
        assert_eq!(
            sid_row.forms,
            vec!["foo".to_string()],
            "sid row took the candidate"
        );
        assert_eq!(sid_row.status, Status::Active);
        let sig_row = lock.rows().find(|r| r.suffix == 1).unwrap();
        assert_eq!(sig_row.status, Status::Tombstone);
    }

    #[test]
    fn same_tier_double_claim_is_a_hard_error() {
        // Pathological (hand-built) lock: two sid-tier rows that both exactly
        // claim one candidate. Ambiguity must never resolve silently.
        let mut lock = Lock::new();
        lock.insert_row(row_s("w", 1, "sid:en:s", "en:s", &["f1"]));
        lock.insert_row(row_s("w", 2, "sid:en:s#sig:f1", "en:s", &["f1"]));
        let err = lock
            .resolve("w", Pos::Noun, vec![cand_s(&["f1"], "en:s")], false, "d1")
            .unwrap_err();
        assert!(err.contains("AMBIGUOUS"), "{err}");
        assert!(err.contains("overrides.csv"), "{err}");
    }

    #[test]
    fn shared_sid_requires_content_match() {
        // Once several active rows hold one sid, sid matching degrades to exact
        // content matching — drift would otherwise be assignable to either sibling.
        let mut lock = Lock::new();
        lock.resolve("w", Pos::Noun, vec![cand_s(&["f1"], "en:s")], false, "d1")
            .unwrap(); // sid:en:s @1
        lock.resolve(
            "w",
            Pos::Noun,
            vec![cand_s(&["f1"], "en:s"), cand_s(&["f2"], "en:s")],
            false,
            "d2",
        )
        .unwrap(); // duplicate sid: @1 keeps f1 (sig agrees), f2 appends composite @2
        assert!(
            lock.rows().any(|r| r.anchor == "sid:en:s#sig:f2"),
            "duplicate sid must freeze a composite anchor"
        );
        // d3: only the f2 variant survives. The bare row must NOT capture it by
        // sid drift (two active rows share the sid); content decides.
        let a = lock
            .resolve("w", Pos::Noun, vec![cand_s(&["f2"], "en:s")], false, "d3")
            .unwrap();
        assert_eq!(keys(&a), vec!["w_2".to_string()]);
        let bare = lock.rows().find(|r| r.suffix == 1).unwrap();
        assert_eq!(bare.status, Status::Tombstone);
        assert_eq!(bare.forms, vec!["f1".to_string()]);
    }

    #[test]
    fn override_rematches_and_waives_immutability() {
        let mut lock = Lock::new();
        lock.resolve("w", Pos::Noun, vec![cand(&["foo"])], false, "d1")
            .unwrap();
        let dir = std::env::temp_dir().join("english_registry_test_override");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("noun.lock.csv");
        lock.save(&path).unwrap();
        let before = Lock::load(&path).unwrap();

        let ov = Overrides {
            rows: vec![Override {
                lemma: "w".into(),
                pos: Pos::Noun,
                suffix: 1,
                old_anchor: "sig:foo".into(),
                new_anchor: "sig:bar".into(),
            }],
        };
        lock.set_overrides(&ov);
        let a = lock
            .resolve("w", Pos::Noun, vec![cand(&["bar"])], false, "d2")
            .unwrap();
        assert_eq!(
            keys(&a),
            vec!["w".to_string()],
            "override carries the key forward"
        );
        let r = lock.rows().find(|r| r.suffix == 1).unwrap();
        assert_eq!(r.anchor, "sig:bar");
        assert_eq!(r.forms, vec!["bar".to_string()]);
        assert!(lock.notes.iter().any(|n| n.contains("override applied")));

        // The override waives exactly this transition — and only with it present.
        assert!(check_immutability(&before, &lock, &ov).is_empty());
        assert!(!check_immutability(&before, &lock, &no_ov()).is_empty());
        // Post-refresh, the override must be in the applied state.
        assert!(ov.check(&lock, Pos::Noun, true).is_empty());
    }

    #[test]
    fn applied_override_must_still_publish_an_active_row() {
        let mut before = Lock::new();
        before
            .resolve("w", Pos::Noun, vec![cand(&["foo"])], false, "d1")
            .unwrap();
        let ov = Overrides {
            rows: vec![Override {
                lemma: "w".into(),
                pos: Pos::Noun,
                suffix: 1,
                old_anchor: "sig:foo".into(),
                new_anchor: "sig:bar".into(),
            }],
        };
        let mut after = Lock::new();
        after.insert_row(LockRow {
            lemma: "w".into(),
            pos: Pos::Noun,
            suffix: 1,
            anchor: "sig:bar".into(),
            qid: None,
            sid: None,
            etym: None,
            status: Status::Tombstone,
            first_seen: "d1".into(),
            last_seen: "d2".into(),
            forms: vec!["bar".into()],
            gloss: None,
        });

        let stale = ov.check(&after, Pos::Noun, true);
        assert!(
            stale
                .iter()
                .any(|m| m.contains("applied anchor") && m.contains("tombstoned")),
            "{stale:?}"
        );
        let violations = check_immutability(&before, &after, &ov);
        assert!(
            violations.iter().any(|m| m.contains("DROPPED")),
            "a tombstoned new_anchor must not waive the dropped published key: {violations:?}"
        );
    }

    #[test]
    fn validate_rejects_duplicate_anchors_and_suffixes() {
        let mut lock = Lock::new();
        for (suffix, anchor, form) in [(1, "sig:a", "a"), (2, "sig:a", "a"), (2, "sig:b", "b")] {
            lock.insert_row(LockRow {
                lemma: "w".into(),
                pos: Pos::Noun,
                suffix,
                anchor: anchor.into(),
                qid: None,
                sid: None,
                etym: None,
                status: Status::Active,
                first_seen: "d1".into(),
                last_seen: "d1".into(),
                forms: vec![form.into()],
                gloss: None,
            });
        }
        let violations = lock.validate();
        assert!(
            violations
                .iter()
                .any(|m| m.contains("anchor `sig:a` is shared by 2 rows")),
            "{violations:?}"
        );
        assert!(
            violations
                .iter()
                .any(|m| m.contains("suffix 2 is held by 2 identities")),
            "{violations:?}"
        );
    }

    #[test]
    fn override_check_flags_stale_rows() {
        let mut lock = Lock::new();
        lock.resolve("w", Pos::Noun, vec![cand(&["foo"])], false, "d1")
            .unwrap();
        let mk = |lemma: &str, suffix: u32, old: &str, new: &str| Overrides {
            rows: vec![Override {
                lemma: lemma.into(),
                pos: Pos::Noun,
                suffix,
                old_anchor: old.into(),
                new_anchor: new.into(),
            }],
        };
        // No such row.
        let v = mk("nope", 1, "sig:a", "sig:b").check(&lock, Pos::Noun, false);
        assert!(v.iter().any(|m| m.contains("no such lock row")), "{v:?}");
        // Anchor matches neither side.
        let v = mk("w", 1, "sig:other", "sig:b").check(&lock, Pos::Noun, false);
        assert!(v.iter().any(|m| m.contains("matches neither")), "{v:?}");
        // Pending is fine before a refresh, an error after one.
        let pending = mk("w", 1, "sig:foo", "sig:bar");
        assert!(pending.check(&lock, Pos::Noun, false).is_empty());
        let v = pending.check(&lock, Pos::Noun, true);
        assert!(v.iter().any(|m| m.contains("still pending")), "{v:?}");

        // A pending override whose target row is TOMBSTONED can never apply
        // (application only targets active rows): committing the proposal-emitting
        // refresh as-is and stacking the override on top must fail check-registry
        // NOW, not the next refresh.
        lock.resolve("w", Pos::Noun, vec![cand(&["bar"])], false, "d2")
            .unwrap(); // sig:foo @1 tombstones, sig:bar appends @2
        let v = pending.check(&lock, Pos::Noun, false);
        assert!(
            v.iter()
                .any(|m| m.contains("tombstoned") && m.contains("git restore")),
            "{v:?}"
        );
    }

    #[test]
    fn ambiguous_drift_tombstones_appends_and_proposes() {
        let mut lock = Lock::new();
        lock.resolve(
            "z",
            Pos::Noun,
            vec![cand(&["p1"]), cand(&["q1"])],
            false,
            "d1",
        )
        .unwrap();
        let dir = std::env::temp_dir().join("english_registry_test_ambig");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("noun.lock.csv");
        lock.save(&path).unwrap();
        let before = Lock::load(&path).unwrap();

        let asg = lock
            .resolve(
                "z",
                Pos::Noun,
                vec![cand(&["p2"]), cand(&["q2"])],
                false,
                "d2",
            )
            .unwrap();
        assert_eq!(keys(&asg), vec!["z_3".to_string(), "z_4".to_string()]);
        assert!(check_immutability(&before, &lock, &no_ov()).is_empty());
        assert!(lock.notes.iter().any(|n| n.contains("AMBIGUOUS")));
        // Every (retired row, leftover candidate) pairing is proposed for review.
        assert_eq!(lock.proposals.len(), 4);
    }

    #[test]
    fn soft_sense_never_takes_bare_key_at_cold_start() {
        // The soft sense carries the LOWER etym (and would win an anchor-first
        // sort); standard-before-soft must still hold.
        let mut lock = Lock::new();
        let mut soft = cand_e(&["badder"], 1);
        soft.soft_sense = true;
        let std_sib = cand_e(&["worse"], 2);
        let a = lock
            .resolve("bad", Pos::Adj, vec![soft, std_sib], false, "d1")
            .unwrap();
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(
            by_key["bad"],
            vec!["worse".to_string()],
            "the standard sibling takes the bare key"
        );
        assert_eq!(by_key["bad_2"], vec!["badder".to_string()]);
    }

    #[test]
    fn cold_start_orders_by_anchor_before_forms() {
        // Among standard senses, anchors (etym here) outrank spelling.
        let mut lock = Lock::new();
        let a = lock
            .resolve(
                "w",
                Pos::Noun,
                vec![cand_e(&["aaa"], 2), cand_e(&["zzz"], 1)],
                false,
                "d1",
            )
            .unwrap();
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(
            by_key["w"],
            vec!["zzz".to_string()],
            "etym 1 takes the bare key"
        );
        assert_eq!(by_key["w_2"], vec!["aaa".to_string()]);
    }

    #[test]
    fn etym_renumber_does_not_swap_sig_frozen_rows() {
        // etym is informational only: a renumber + drift tombstones and appends,
        // never re-pairs.
        let mut lock = Lock::new();
        lock.resolve(
            "x",
            Pos::Noun,
            vec![cand(&["alpha"]), cand(&["beta"])],
            false,
            "d1",
        )
        .unwrap();
        lock.resolve(
            "x",
            Pos::Noun,
            vec![cand_e(&["alpha"], 1), cand_e(&["beta"], 2)],
            false,
            "d2",
        )
        .unwrap();
        let dir = std::env::temp_dir().join("english_registry_test_etymswap");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("noun.lock.csv");
        lock.save(&path).unwrap();
        let before = Lock::load(&path).unwrap();

        // d3: etyms RENUMBERED (alpha now 2, beta now 1) AND both forms drift.
        lock.resolve(
            "x",
            Pos::Noun,
            vec![cand_e(&["alpha2"], 2), cand_e(&["beta2"], 1)],
            false,
            "d3",
        )
        .unwrap();
        let by_anchor: BTreeMap<String, &LockRow> =
            lock.rows().map(|r| (r.anchor.clone(), r)).collect();
        let a = by_anchor.get("sig:alpha").expect("alpha row");
        let b = by_anchor.get("sig:beta").expect("beta row");
        assert_eq!(a.suffix, 1);
        assert_eq!(b.suffix, 2);
        assert_eq!(
            a.forms,
            vec!["alpha".to_string()],
            "alpha must NOT inherit beta's forms"
        );
        assert_eq!(
            b.forms,
            vec!["beta".to_string()],
            "beta must NOT inherit alpha's forms"
        );
        assert_eq!(a.status, Status::Tombstone);
        assert_eq!(b.status, Status::Tombstone);
        let v = check_immutability(&before, &lock, &no_ov());
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn tombstone_reappear_gets_unique_anchor() {
        // A sense that vanished (tombstoned) then reappears re-derives the same
        // anchor; the new row must get a UNIQUE anchor (not collide with its own
        // tombstone) and a strictly-higher suffix.
        let mut lock = Lock::new();
        lock.resolve("x", Pos::Noun, vec![cand(&["alpha"])], false, "d1")
            .unwrap();
        lock.resolve("x", Pos::Noun, vec![], false, "d2").unwrap(); // vanishes
        let a = lock
            .resolve("x", Pos::Noun, vec![cand(&["alpha"])], false, "d3")
            .unwrap();
        assert_eq!(
            keys(&a),
            vec!["x_2".to_string()],
            "reappearing sense gets a new suffix"
        );
        let anchors: Vec<String> = lock.rows().map(|r| r.anchor.clone()).collect();
        let distinct: std::collections::HashSet<_> = anchors.iter().collect();
        assert_eq!(
            anchors.len(),
            distinct.len(),
            "anchors must stay unique: {anchors:?}"
        );
        // Internal consistency (unique anchors/suffixes) is what a self-comparison
        // can attest; assert it directly rather than through the cross-state guard.
        assert!(lock.validate().is_empty());
    }

    #[test]
    fn migrate_rewrites_legacy_tiers_to_sig() {
        let mut lock = Lock::new();
        lock.insert_row(row("a", 1, "qid:Q1", Some("Q1"), &["ax"]));
        lock.insert_row(row("b", 1, "etym:2#sig:bx", None, &["bx"]));
        lock.insert_row(row_s("c", 1, "sid:en:c", "en:c", &["cx"]));
        lock.insert_row(row("d", 1, "sig:dx", None, &["dx"]));
        // Same lemma as an existing sig row -> needs the #u disambiguator.
        let mut dup = row("d", 2, "qid:Q7", Some("Q7"), &["dx"]);
        dup.status = Status::Tombstone;
        lock.insert_row(dup);

        let migrated = lock.migrate_to_exact_anchors();
        assert_eq!(migrated, 3);
        let anchor_of = |lemma: &str, suffix: u32| {
            lock.rows()
                .find(|r| r.lemma == lemma && r.suffix == suffix)
                .unwrap()
                .anchor
                .clone()
        };
        assert_eq!(anchor_of("a", 1), "sig:ax");
        assert_eq!(anchor_of("b", 1), "sig:bx");
        assert_eq!(anchor_of("c", 1), "sid:en:c", "sid rows keep their anchor");
        assert_eq!(anchor_of("d", 1), "sig:dx", "sig rows keep their anchor");
        assert_eq!(
            anchor_of("d", 2),
            "sig:dx#u2",
            "collisions disambiguate by suffix"
        );
        assert!(lock.validate().is_empty(), "{:?}", lock.validate());
    }

    #[test]
    fn reap_tombstones_absent_lemma() {
        let mut lock = Lock::new();
        lock.resolve("alpha", Pos::Noun, vec![cand(&["alphae"])], false, "d1")
            .unwrap();
        lock.resolve("beta", Pos::Noun, vec![cand(&["betae"])], false, "d1")
            .unwrap();
        let mut resolved = std::collections::HashSet::new();
        resolved.insert("alpha".to_string());
        let rep = lock.reap(Pos::Noun, &resolved, "d2", 0.90);
        assert_eq!(rep.tombstoned, 1);
        assert!(!rep.skipped);
        let beta = lock.rows().find(|r| r.lemma == "beta").unwrap();
        assert_eq!(beta.status, Status::Tombstone);
        let alpha = lock.rows().find(|r| r.lemma == "alpha").unwrap();
        assert_eq!(alpha.status, Status::Active);
        let emitted: Vec<String> = lock
            .emittable(Pos::Noun)
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert!(emitted.contains(&"alpha".to_string()));
        assert!(
            !emitted.contains(&"beta".to_string()),
            "reaped key must stop being emitted"
        );
    }

    #[test]
    fn reap_skips_on_mass_absence() {
        let mut lock = Lock::new();
        lock.resolve("a", Pos::Noun, vec![cand(&["airr"])], false, "d1")
            .unwrap();
        lock.resolve("b", Pos::Noun, vec![cand(&["birr"])], false, "d1")
            .unwrap();
        lock.resolve("c", Pos::Noun, vec![cand(&["cirr"])], false, "d1")
            .unwrap();
        lock.resolve("d", Pos::Noun, vec![cand(&["dirr"])], false, "d1")
            .unwrap();
        let mut resolved = std::collections::HashSet::new();
        resolved.insert("a".to_string());
        let rep = lock.reap(Pos::Noun, &resolved, "d2", 0.10);
        assert!(rep.skipped);
        assert_eq!(rep.tombstoned, 0);
        assert!(lock.rows().all(|r| r.status == Status::Active));
    }

    fn row(lemma: &str, suffix: u32, anchor: &str, qid: Option<&str>, forms: &[&str]) -> LockRow {
        LockRow {
            lemma: lemma.to_string(),
            pos: Pos::Noun,
            suffix,
            anchor: anchor.to_string(),
            qid: qid.map(|s| s.to_string()),
            sid: None,
            etym: None,
            status: Status::Active,
            first_seen: "d".to_string(),
            last_seen: "d".to_string(),
            forms: forms.iter().map(|s| s.to_string()).collect(),
            gloss: None,
        }
    }

    fn row_s(lemma: &str, suffix: u32, anchor: &str, sid: &str, forms: &[&str]) -> LockRow {
        let mut r = row(lemma, suffix, anchor, None, forms);
        r.sid = Some(sid.to_string());
        r
    }

    #[test]
    fn anchor_parse_roundtrips_every_shape() {
        // The typed Anchor must serialize byte-identically to every stored shape,
        // including a `#`-bearing senseid (real dump data: `en:totality#English`),
        // which the old first-`#` split silently truncated.
        for s in [
            "sig:foo",
            "sig:more free|most free", // forms carry spaces and the `|` separator
            "sig:foo#u3",
            "sid:en:muscle",
            "sid:en:muscle#u7",
            "sid:en:Christianity#sig:agapae",
            "sid:en:Christianity#sig:agapae#u2",
            "sid:en:totality#English", // '#' inside the senseid must not truncate
        ] {
            let a = Anchor::parse(s).unwrap_or_else(|| panic!("failed to parse {s}"));
            assert_eq!(a.to_string(), s, "round-trip mismatch for {s}");
        }
        // The '#'-bearing senseid keeps its full value and pins no forms.
        let a = Anchor::parse("sid:en:totality#English").unwrap();
        assert_eq!(a.senseid(), Some("en:totality#English"));
        assert!(a.frozen_sig().is_none());
        assert!(a.allows_form_drift());
        // A content-qualified sid pins its forms and does not drift.
        let q = Anchor::parse("sid:en:c#sig:cs").unwrap();
        assert_eq!(q.senseid(), Some("en:c"));
        assert_eq!(q.frozen_sig(), Some("cs"));
        assert!(!q.allows_form_drift());
        // Legacy tiers do not parse.
        assert!(Anchor::parse("qid:Q1").is_none());
        assert!(Anchor::parse("etym:2").is_none());
    }

    #[test]
    fn override_refuses_ambiguous_bare_sid_match() {
        // A bare-sid override names a sense by senseid ALONE. When two candidates
        // carry that senseid (variant plurals), the override must not positionally
        // pick one — it refuses (stays pending) so the ambiguity is reviewed.
        // Regression: the old candidate_matches_anchor picked the first in dump order.
        let mut lock = Lock::new();
        lock.insert_row(row("w", 1, "sig:foo", None, &["foo"]));
        let ov = Overrides {
            rows: vec![Override {
                lemma: "w".into(),
                pos: Pos::Noun,
                suffix: 1,
                old_anchor: "sig:foo".into(),
                new_anchor: "sid:en:bison".into(),
            }],
        };
        lock.set_overrides(&ov);
        lock.resolve(
            "w",
            Pos::Noun,
            vec![
                cand_s(&["buffalo"], "en:bison"),
                cand_s(&["buffaloes"], "en:bison"),
            ],
            false,
            "d2",
        )
        .unwrap();
        let r1 = lock.rows().find(|r| r.suffix == 1).unwrap();
        assert_eq!(r1.anchor, "sig:foo", "override must not have applied");
        assert!(
            lock.notes
                .iter()
                .any(|n| n.contains("NOT applied") && n.contains("sibling")),
            "{:?}",
            lock.notes
        );
    }

    #[test]
    fn override_does_not_rob_a_content_sibling() {
        // Committed: w=sig:foo[foo], w_2=sig:bar[bar]. An override re-anchors w to
        // sid:en:x; the dump's en:x candidate has forms [bar], which w_2 exactly
        // content-claims. Applying would silently drop w_2 — the override must refuse.
        // Regression: the old senseid-only match consumed [bar] before the content
        // matrix ran, tombstoning w_2 and mutating w.
        let mut lock = Lock::new();
        lock.insert_row(row("w", 1, "sig:foo", None, &["foo"]));
        lock.insert_row(row("w", 2, "sig:bar", None, &["bar"]));
        let ov = Overrides {
            rows: vec![Override {
                lemma: "w".into(),
                pos: Pos::Noun,
                suffix: 1,
                old_anchor: "sig:foo".into(),
                new_anchor: "sid:en:x".into(),
            }],
        };
        lock.set_overrides(&ov);
        lock.resolve("w", Pos::Noun, vec![cand_s(&["bar"], "en:x")], false, "d2")
            .unwrap();
        let r1 = lock.rows().find(|r| r.suffix == 1).unwrap();
        assert_eq!(r1.anchor, "sig:foo", "override must not have applied");
        let r2 = lock.rows().find(|r| r.suffix == 2).unwrap();
        assert_eq!(
            r2.forms,
            vec!["bar".to_string()],
            "sibling kept its content"
        );
        assert_eq!(r2.status, Status::Active, "sibling must not be robbed");
        assert!(
            lock.notes
                .iter()
                .any(|n| n.contains("NOT applied") && n.contains("content match")),
            "{:?}",
            lock.notes
        );
    }

    #[test]
    fn override_consumed_twin_still_gates_sid_drift() {
        // Two ACTIVE rows share senseid en:x in the committed bucket (a bare row and
        // a #u reappearance twin). An override re-anchors the bare one; the drift
        // gate on the twin must still see en:x as shared in the COMMITTED bucket, so
        // a same-sid different-content candidate tombstones+appends instead of
        // silently drifting the twin. Regression: sid sharing was computed over the
        // post-override matched set, so consuming the bare row loosened the twin's gate.
        let mut lock = Lock::new();
        lock.insert_row(row_s("w", 1, "sid:en:x", "en:x", &["fa"]));
        lock.insert_row(row_s("w", 5, "sid:en:x#u5", "en:x", &["fb"]));
        let ov = Overrides {
            rows: vec![Override {
                lemma: "w".into(),
                pos: Pos::Noun,
                suffix: 1,
                old_anchor: "sid:en:x".into(),
                new_anchor: "sig:fz".into(),
            }],
        };
        lock.set_overrides(&ov);
        lock.resolve(
            "w",
            Pos::Noun,
            vec![cand(&["fz"]), cand_s(&["fb2"], "en:x")],
            false,
            "d2",
        )
        .unwrap();
        // The bare row took the override candidate.
        let r1 = lock.rows().find(|r| r.suffix == 1).unwrap();
        assert_eq!(r1.anchor, "sig:fz");
        assert_eq!(r1.forms, vec!["fz".to_string()]);
        // The twin must NOT have drifted onto fb2; it retired instead.
        let r5 = lock.rows().find(|r| r.suffix == 5).unwrap();
        assert_eq!(r5.forms, vec!["fb".to_string()], "twin must not drift");
        assert_eq!(r5.status, Status::Tombstone);
    }

    #[test]
    fn emit_collisions_detects_duplicate_keys() {
        let mut lock = Lock::new();
        lock.insert_row(row("w", 1, "sig:a", None, &["a"]));
        lock.insert_row(row("w", 0, "sig:b", None, &["b"]));
        let c = lock.emit_collisions(Pos::Noun);
        assert_eq!(c.len(), 1, "{c:?}");
        assert!(c[0].contains("emit key `w`"), "{c:?}");
    }

    #[test]
    fn validate_flags_suffix_zero_and_legacy_tiers() {
        let mut lock = Lock::new();
        lock.insert_row(row("z", 0, "sig:z", None, &["z"])); // suffix 0
        lock.insert_row(row("q", 2, "qid:Q1", Some("Q1"), &["q"])); // legacy tier
        lock.insert_row(row("e", 2, "etym:1", None, &["e"])); // legacy tier
        let v = lock.validate();
        assert!(v.iter().any(|m| m.contains("suffix 0")), "{v:?}");
        assert_eq!(
            v.iter().filter(|m| m.contains("unknown tier")).count(),
            2,
            "legacy qid/etym anchors must be rejected until migrated: {v:?}"
        );
        // sid-tier with empty sid field is inconsistent.
        let mut bad_sid = row("s", 2, "sid:en:x", None, &["s"]);
        bad_sid.sid = None;
        let mut lock2 = Lock::new();
        lock2.insert_row(bad_sid);
        let v2 = lock2.validate();
        assert!(
            v2.iter()
                .any(|m| m.contains("sid-tier but the sid field is empty")),
            "{v2:?}"
        );
        // A clean lock validates empty.
        let mut ok = Lock::new();
        ok.insert_row(row("good", 2, "sig:goods", None, &["goods"]));
        ok.insert_row(row_s("fine", 2, "sid:en:f", "en:f", &["fines"]));
        assert!(ok.validate().is_empty(), "{:?}", ok.validate());
    }

    #[test]
    fn deleting_a_tombstone_row_is_a_violation() {
        let mut old = Lock::new();
        old.insert_row(row("x", 1, "sig:a", None, &["a"]));
        let mut tomb = row("x", 2, "sig:b", None, &["b"]);
        tomb.status = Status::Tombstone;
        old.insert_row(tomb);

        let mut new = Lock::new();
        new.insert_row(row("x", 1, "sig:a", None, &["a"]));

        let v = check_immutability(&old, &new, &no_ov());
        assert!(
            v.iter()
                .any(|m| m.contains("tombstone") && m.contains("DROPPED")),
            "{v:?}"
        );
    }

    #[test]
    fn validate_flags_bad_forms_arity_and_empty_columns() {
        let mut lock = Lock::new();
        let mut short = row("v", 2, "sig:a|b", None, &["a", "b"]);
        short.pos = Pos::Verb;
        lock.insert_row(short);
        lock.insert_row(row("n", 2, "sig:", None, &[""]));
        let mut tomb = row("t", 2, "sig:x", None, &[]);
        tomb.status = Status::Tombstone;
        lock.insert_row(tomb);

        let v = lock.validate();
        assert!(
            v.iter().any(|m| m.contains("expected 4")),
            "verb arity: {v:?}"
        );
        assert!(
            v.iter().any(|m| m.contains("empty form column")),
            "empty column: {v:?}"
        );
        assert!(
            !v.iter().any(|m| m.starts_with("t ")),
            "tombstones exempt: {v:?}"
        );
    }

    #[test]
    fn load_rejects_garbage_etym_but_accepts_empty() {
        let dir = std::env::temp_dir().join("english_registry_test_etymparse");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("noun.lock.csv");
        let header =
            "lemma,pos,suffix,anchor,qid,sid,etym,status,first_seen,last_seen,forms,glosses";
        std::fs::write(
            &path,
            format!("{header}\nw,noun,1,sig:ws,,,,active,d,d,ws,\n"),
        )
        .unwrap();
        let lock = Lock::load(&path).unwrap();
        assert_eq!(lock.rows().next().unwrap().etym, None);

        std::fs::write(
            &path,
            format!("{header}\nw,noun,1,sig:ws,,,one,active,d,d,ws,\n"),
        )
        .unwrap();
        let err = Lock::load(&path).unwrap_err().to_string();
        assert!(err.contains("invalid etym"), "{err}");
    }

    #[test]
    fn split_key_round_trips_make_key() {
        assert_eq!(split_key("die_2"), Some(("die", 2)));
        assert_eq!(split_key("die"), None);
        assert_eq!(split_key("mp3"), None); // no underscore -> opaque
        assert_eq!(split_key("_2"), None); // empty base
        assert_eq!(split_key("x_"), None); // empty digits
        assert_eq!(split_key("x_2a"), None); // non-digit suffix
        for (lemma, suffix) in [("lie", 2u32), ("abide", 3), ("x", 17)] {
            let key = make_key(lemma, suffix);
            assert_eq!(split_key(&key), Some((lemma, suffix)));
        }
    }

    #[test]
    #[should_panic(expected = "overflows u32")]
    fn split_key_overflow_is_a_hard_error() {
        let _ = split_key("foo_4294967296");
    }

    #[test]
    fn reap_respects_max_fraction_boundary() {
        let build = || {
            let mut lock = Lock::new();
            lock.resolve("a", Pos::Noun, vec![cand(&["air"])], false, "d1")
                .unwrap();
            lock.resolve("b", Pos::Noun, vec![cand(&["bir"])], false, "d1")
                .unwrap();
            lock.resolve("c", Pos::Noun, vec![cand(&["cir"])], false, "d1")
                .unwrap();
            lock.resolve("d", Pos::Noun, vec![cand(&["dir"])], false, "d1")
                .unwrap();
            lock
        };
        let mut resolved = std::collections::HashSet::new();
        resolved.insert("a".to_string());
        resolved.insert("b".to_string());
        resolved.insert("c".to_string());

        let mut at = build();
        let rep = at.reap(Pos::Noun, &resolved, "d2", 0.25);
        assert!(!rep.skipped, "25% at the 0.25 boundary should reap");
        assert_eq!(rep.tombstoned, 1);

        let mut over = build();
        let rep = over.reap(Pos::Noun, &resolved, "d2", 0.10);
        assert!(rep.skipped, "25% over the 0.10 ceiling must be refused");
        assert_eq!(rep.tombstoned, 0);
    }

    #[test]
    fn compact_drops_tombstones_and_closes_gaps_in_order() {
        let mut lock = Lock::new();
        lock.resolve(
            "x",
            Pos::Noun,
            vec![cand(&["a"]), cand(&["b"]), cand(&["c"]), cand(&["d"])],
            false,
            "d1",
        )
        .unwrap(); // a=1, b=2, c=3, d=4 (alphabetical cold start)
        lock.resolve(
            "x",
            Pos::Noun,
            vec![cand(&["b"]), cand(&["d"])],
            false,
            "d2",
        )
        .unwrap(); // 1,3 tombstoned

        let report = lock.compact();
        assert_eq!(report.tombstones_dropped, 2);
        let rows: Vec<(u32, String)> = lock.rows().map(|r| (r.suffix, r.sig())).collect();
        assert_eq!(rows, vec![(2, "b".to_string()), (3, "d".to_string())]);
        assert!(lock.validate().is_empty());
    }

    #[test]
    fn compact_keeps_existing_bare_key_and_is_noop_on_dense_lock() {
        let mut lock = Lock::new();
        lock.resolve(
            "y",
            Pos::Noun,
            vec![cand(&["p"]), cand(&["q"])],
            false,
            "d1",
        )
        .unwrap(); // p=1, q=2
        let report = lock.compact();
        assert!(report.is_noop(), "{report:?}");
        let rows: Vec<u32> = lock.rows().map(|r| r.suffix).collect();
        assert_eq!(rows, vec![1, 2], "an already-dense lock is untouched");
    }

    #[test]
    fn compact_removes_fully_tombstoned_lemmas() {
        let mut lock = Lock::new();
        lock.resolve("z", Pos::Noun, vec![cand(&["zz"])], false, "d1")
            .unwrap();
        lock.resolve("z", Pos::Noun, vec![], false, "d2").unwrap(); // vanished
        let report = lock.compact();
        assert_eq!(report.tombstones_dropped, 1);
        assert_eq!(
            lock.rows().count(),
            0,
            "empty bucket must be dropped entirely"
        );
    }

    #[test]
    fn validate_flags_anchor_payload_mismatches() {
        // The frozen payload must name the row's own identity: a sig anchor whose
        // forms drifted in place, a sid anchor disagreeing with the sid column,
        // and a #sig: qualifier disagreeing with the forms are all corruption.
        let mut lock = Lock::new();
        lock.insert_row(row("a", 1, "sig:old", None, &["new"]));
        lock.insert_row(row_s("b", 1, "sid:en:x", "en:OTHER", &["bs"]));
        lock.insert_row(row_s("c", 1, "sid:en:c#sig:cold", "en:c", &["cnew"]));
        let v = lock.validate();
        assert!(
            v.iter()
                .any(|m| m.contains("freezes sig `old`") && m.contains("`new`")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("names sid `en:x`") && m.contains("`en:OTHER`")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("freezes sig `cold`") && m.contains("`cnew`")),
            "{v:?}"
        );

        // Consistent rows — including a #u-disambiguated sig anchor — are clean.
        let mut ok = Lock::new();
        ok.insert_row(row("a", 2, "sig:as", None, &["as"]));
        ok.insert_row(row("a", 3, "sig:as#u3", None, &["as"]));
        ok.insert_row(row_s("b", 2, "sid:en:b", "en:b", &["bs"]));
        ok.insert_row(row_s("c", 2, "sid:en:c#sig:cs", "en:c", &["cs"]));
        assert!(ok.validate().is_empty(), "{:?}", ok.validate());
    }

    #[test]
    fn coverage_buckets_by_frozen_anchor_tier() {
        let mut lock = Lock::new();
        // sig-anchored but carrying informational qid: must count as sig.
        lock.insert_row(row("a", 1, "sig:as", Some("Q1"), &["as"]));
        lock.insert_row(row_s("b", 1, "sid:en:b", "en:b", &["bs"]));
        let mut tomb = row("t", 2, "sig:ts", None, &["ts"]);
        tomb.status = Status::Tombstone;
        lock.insert_row(tomb);
        let c = lock.coverage();
        assert_eq!(
            (c.active, c.by_sid, c.by_sig, c.legacy, c.tombstoned),
            (2, 1, 1, 0, 1)
        );
    }

    #[test]
    fn overrides_load_round_trip_and_strictness() {
        let dir = std::env::temp_dir().join("english_registry_test_ovload");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("overrides.csv");
        std::fs::write(
            &path,
            "lemma,pos,suffix,old_anchor,new_anchor\nw,noun,1,sig:foo,sig:bar\n",
        )
        .unwrap();
        let ov = Overrides::load(&path).unwrap();
        assert_eq!(ov.rows.len(), 1);
        assert_eq!(ov.rows[0].old_anchor, "sig:foo");

        // Missing file -> empty set; malformed row -> hard error.
        assert!(
            Overrides::load(dir.join("absent.csv"))
                .unwrap()
                .rows
                .is_empty()
        );
        std::fs::write(
            &path,
            "lemma,pos,suffix,old_anchor,new_anchor\nw,noun,one,sig:foo,sig:bar\n",
        )
        .unwrap();
        assert!(Overrides::load(&path).is_err());
        std::fs::write(
            &path,
            "lemma,pos,suffix,old_anchor,new_anchor\nw,noun,1,sig:foo,sig:foo\n",
        )
        .unwrap();
        assert!(Overrides::load(&path).is_err(), "old == new is malformed");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// One observed sense: forms plus the metadata tiers (etym / qid / sid). sid
    /// exercises the durable tier (bare, composite, and drift paths); qid/etym
    /// exercise the informational columns and the qid conflict veto. Small
    /// alphabets keep the search dense (and force collisions). `arity` form
    /// columns match the POS the stream resolves as, so `Lock::validate` (asserted
    /// every step) accepts them and the multi-column `forms_sig` packing is fuzzed.
    type Sense = (Vec<String>, Option<u32>, Option<String>, Option<String>);
    /// A refresh stream: per dump, a `had_regular` flag and senses for two lemmas.
    type DumpStream = Vec<(bool, Vec<Sense>, Vec<Sense>)>;
    fn sense(arity: usize) -> impl Strategy<Value = Sense> {
        (
            prop::collection::vec("[a-c]{1,3}", arity..=arity),
            prop::option::of(0u32..3),
            prop::option::of("Q[1-3]"),
            prop::option::of("s[1-3]"),
        )
    }

    /// The POS to resolve a stream as, paired with the form-column arity
    /// `Lock::validate` enforces (noun=1, verb=4, adj=2). Fuzzing all three covers
    /// the multi-column `forms_sig` packing and the per-POS validate arity branch,
    /// not just the single-column noun path.
    fn pos_and_arity() -> impl Strategy<Value = (Pos, usize)> {
        prop_oneof![
            Just((Pos::Noun, 1)),
            Just((Pos::Verb, 4)),
            Just((Pos::Adj, 2)),
        ]
    }

    /// A stream of dumps. Each dump carries an independent `had_regular` flag (to
    /// exercise the cold-start-at-2 branch) and senses for TWO lemmas (to confirm
    /// the per-(lemma,pos) buckets stay independent).
    fn dumps(arity: usize) -> impl Strategy<Value = DumpStream> {
        let dump = (
            any::<bool>(),
            prop::collection::vec(sense(arity), 0..3usize),
            prop::collection::vec(sense(arity), 0..3usize),
        );
        prop::collection::vec(dump, 1..6usize)
    }

    /// A POS plus a dump stream whose senses carry that POS's form arity.
    fn pos_stream() -> impl Strategy<Value = (Pos, DumpStream)> {
        pos_and_arity().prop_flat_map(|(pos, arity)| (Just(pos), dumps(arity)))
    }

    /// Keep the smaller of the existing and incoming value — the same
    /// order-independent merge `LemmaAcc::observe` uses in the extract layer, so
    /// the strategy feeds resolve() the anchor combinations the pipeline really
    /// produces (min-merged across same-signature senses).
    fn merge_min<T: Ord>(slot: &mut Option<T>, incoming: Option<T>) {
        if let Some(new) = incoming {
            match slot {
                Some(cur) if *cur <= new => {}
                _ => *slot = Some(new),
            }
        }
    }

    fn to_candidates(senses: &[Sense]) -> Vec<Candidate> {
        let mut by_sig: BTreeMap<String, Candidate> = BTreeMap::new();
        for (forms, etym, qid, sid) in senses {
            let c = by_sig
                .entry(forms_sig(forms))
                .or_insert_with(|| Candidate::new(forms.clone()));
            merge_min(&mut c.etym, *etym);
            merge_min(&mut c.qid, qid.clone());
            merge_min(&mut c.sid, sid.clone());
        }
        by_sig.into_values().collect()
    }

    /// A comparable snapshot of a lock's full row state.
    fn snapshot(lock: &Lock) -> Vec<(String, String, u32, bool, Vec<String>)> {
        let mut s: Vec<_> = lock
            .rows()
            .map(|r| {
                (
                    r.lemma.clone(),
                    r.anchor.clone(),
                    r.suffix,
                    r.status == Status::Active,
                    r.forms.clone(),
                )
            })
            .collect();
        s.sort();
        s
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1024))]

        /// Across an arbitrary stream of refreshes — over every POS's form arity,
        /// sid/sig anchors, informational qid/etym, both had_regular branches, and
        /// two independent lemmas — resolution never errors, no published (active)
        /// anchor ever changes its suffix, no tombstoned suffix is revived/reused,
        /// and the entire outcome is invariant under permutation of the candidate
        /// input order. Verified by check_immutability between consecutive on-disk
        /// states, an independent (lemma, anchor) -> suffix monotonicity ledger,
        /// and a mirrored resolve over reversed candidates. Exercises the real
        /// resolve -> save -> load cycle.
        #[test]
        fn published_keys_never_move_or_revive((pos, dumps) in pos_stream()) {
            let dir = std::env::temp_dir().join(format!("english_proptest_lock_{}", std::process::id()));
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join(format!("{}.lock.csv", pos.as_str()));

            let mut lock = Lock::new();
            let mut seen: std::collections::HashMap<(String, String), u32> = Default::default();

            for (i, (had_regular, sa, sb)) in dumps.iter().enumerate() {
                for (lemma, senses) in [("w", sa), ("x", sb)] {
                    let candidates = to_candidates(senses);

                    lock.save(&path).unwrap();
                    let before = Lock::load(&path).unwrap();

                    // Mirror: same dump, reversed candidate order, on a cloned lock.
                    let mut mirror = lock.clone();
                    let mut reversed = candidates.clone();
                    reversed.reverse();

                    let res = lock.resolve(lemma, pos, candidates, *had_regular, &format!("d{i}"));
                    prop_assert!(res.is_ok(), "resolve errored: {:?}", res.err());
                    let mres = mirror.resolve(lemma, pos, reversed, *had_regular, &format!("d{i}"));
                    prop_assert!(mres.is_ok());
                    prop_assert_eq!(snapshot(&lock), snapshot(&mirror), "candidate order changed the outcome");

                    // No published key changed meaning between the two on-disk states.
                    let no_ov = Overrides::default();
                    prop_assert!(check_immutability(&before, &lock, &no_ov).is_empty());
                    // The lock is always internally valid.
                    prop_assert!(lock.validate().is_empty());

                    // (lemma, anchor) -> suffix is monotone for the whole stream.
                    for r in lock.rows() {
                        let key = (r.lemma.clone(), r.anchor.clone());
                        if let Some(&s) = seen.get(&key) {
                            prop_assert_eq!(s, r.suffix);
                        } else {
                            seen.insert(key, r.suffix);
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}
