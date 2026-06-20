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
//! a **frozen suffix**. On every refresh, [`Lock::resolve`] matches the dump's
//! candidate senses to existing lock rows by stable anchors, reuses their pinned
//! suffixes (updating only the stored forms), allocates strictly-higher suffixes
//! for genuinely new senses, and tombstones senses that disappeared (their suffix
//! is retired forever, never reassigned). [`check_immutability`] turns "a
//! published key changed meaning" into a hard, reviewable build failure.
//!
//! Anchor strength, strongest first:
//! 1. `qid:Q####` — Wikidata QID (globally stable external id).
//! 2. `sid:en:...` — author-authored senseid (durable by Wiktionary convention).
//! 3. `etym:<n>` — etymology_number (per-entry; stable under form/tag edits).
//! 4. `etym:<n>#sig:<forms>` — when an etymology has >1 emitted sense, the form
//!    signature breaks the tie (the only place forms enter identity).
//! 5. `sig:<forms>` — last-resort content anchor (used for rows bootstrapped from
//!    the pre-existing tables, where no dump metadata was available).

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
}

impl Candidate {
    pub fn new(forms: Vec<String>) -> Self {
        Candidate {
            forms,
            ..Candidate::default()
        }
    }
    fn sig(&self) -> String {
        self.forms.join("|")
    }
    /// A *deterministic, reproducible* total order for assigning suffixes to the
    /// brand-new identities that co-appear at cold start. It orders by the anchor
    /// fields (qid, then sid, then etym) and only then the form signature, so the
    /// assignment doesn't rest solely on an alphabetical-forms artifact when stronger
    /// metadata is present. Note this is purely a tiebreak among co-appearing new
    /// senses (the result is frozen immediately after) — NOT a claim about which
    /// sense is "primary": a missing qid sorts as the empty string, so a
    /// metadata-less sense can sort ahead of a metadata-bearing one. The bare-vs-`2`
    /// choice among genuine homographs is intentionally arbitrary-but-stable.
    fn order_key(&self) -> (String, String, u32, String) {
        (
            self.qid.clone().unwrap_or_default(),
            self.sid.clone().unwrap_or_default(),
            self.etym.unwrap_or(0),
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
    /// this; it never changes once written.
    pub anchor: String,
    /// Live anchors, enriched over time as stronger metadata becomes available.
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
        self.forms.join("|")
    }

    /// Whether this row's *frozen* identity is its etymology number. Only such rows
    /// may be re-matched on etym alone: `etymology_number` is a renumberable section
    /// ordinal, so trusting it for a row whose frozen identity is something else
    /// (a form signature) would let an upstream etymology reorder silently swap two
    /// senses' forms. qid/sid are durable and need no such guard.
    fn is_etym_frozen(&self) -> bool {
        self.anchor.starts_with("etym:")
    }

    /// Fold a freshly-matched candidate into this row: refresh the forms, enrich
    /// any missing anchors, and bump `last_seen`. The frozen `anchor` and `suffix`
    /// are deliberately untouched.
    fn apply_match(&mut self, c: &Candidate, date: &str) {
        if self.qid.is_none() {
            self.qid = c.qid.clone();
        }
        if self.sid.is_none() {
            self.sid = c.sid.clone();
        }
        if self.etym.is_none() {
            self.etym = c.etym;
        }
        if c.gloss.is_some() {
            self.gloss = c.gloss.clone();
        }
        self.forms = c.forms.clone();
        self.last_seen = date.to_string();
        self.status = Status::Active;
    }
}

/// The output of resolving one lemma: a stable key plus the forms to emit under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub key: String,
    pub forms: Vec<String>,
}

/// An entire lockfile (all rows for one part of speech), indexed by `(lemma, pos)`.
#[derive(Debug, Default)]
pub struct Lock {
    map: BTreeMap<(String, String), Vec<LockRow>>,
    /// Human-facing notes accumulated during resolution (drift matches, tombstones,
    /// ambiguous cases) — surfaced to the maintainer, never affects determinism.
    pub notes: Vec<String>,
}

impl Lock {
    pub fn new() -> Self {
        Lock::default()
    }

    /// Insert a fully-formed row (used by the bootstrap path).
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
    /// bare key), a frozen anchor whose tier field is empty, an unknown anchor tier,
    /// and any emit-key collision. Returns human-readable violations; empty = valid.
    pub fn validate(&self) -> Vec<String> {
        let mut v = Vec::new();
        for r in self.rows() {
            if r.suffix < 1 {
                v.push(format!(
                    "{} [{}] has suffix {} (must be >= 1)",
                    r.lemma,
                    r.pos.as_str(),
                    r.suffix
                ));
            }
            let tier = r.anchor.split(':').next().unwrap_or("");
            match tier {
                "qid" if r.qid.is_none() => v.push(format!(
                    "{} [{}] anchor `{}` is qid-tier but the qid field is empty",
                    r.lemma, r.pos.as_str(), r.anchor
                )),
                "sid" if r.sid.is_none() => v.push(format!(
                    "{} [{}] anchor `{}` is sid-tier but the sid field is empty",
                    r.lemma, r.pos.as_str(), r.anchor
                )),
                "etym" if r.etym.is_none() => v.push(format!(
                    "{} [{}] anchor `{}` is etym-tier but the etym field is empty",
                    r.lemma, r.pos.as_str(), r.anchor
                )),
                "qid" | "sid" | "etym" | "sig" => {}
                other => v.push(format!(
                    "{} [{}] anchor `{}` has unknown tier `{}`",
                    r.lemma, r.pos.as_str(), r.anchor, other
                )),
            }
        }
        for pos in [Pos::Adj, Pos::Noun, Pos::Verb] {
            v.extend(self.emit_collisions(pos));
        }
        v.sort();
        v
    }

    /// Resolve one lemma's candidates against the committed lock, mutating the lock
    /// in place and returning the stable assignments. `had_regular` records whether
    /// a regular-prediction-equal sense was seen for this lemma (and dropped by the
    /// caller as a runtime fall-through), which decides cold-start numbering.
    pub fn resolve(
        &mut self,
        lemma: &str,
        pos: Pos,
        candidates: Vec<Candidate>,
        had_regular: bool,
        date: &str,
    ) -> Vec<Assignment> {
        let mut notes = Vec::new();
        let key = (lemma.to_string(), pos.as_str().to_string());

        // etymology_number is only a safe stand-alone anchor when distinct across
        // the batch; otherwise colliding senses need the form signature as a tiebreak.
        let uniq = BatchUniq::of(&candidates);

        let assignments = {
            let rows = self.map.entry(key).or_default();
            let original_len = rows.len();
            let mut matched = vec![false; original_len];
            let mut remaining = candidates;
            let mut assignments: Vec<Assignment> = Vec::new();

            // -- Phase A: exact anchor match, rows in suffix order --
            for ri in 0..original_len {
                if rows[ri].status != Status::Active {
                    matched[ri] = true; // tombstones are inert
                    continue;
                }
                if let Some(ci) = remaining
                    .iter()
                    .position(|c| anchor_exact_match(&rows[ri], c, &uniq))
                {
                    let c = remaining.remove(ci);
                    rows[ri].apply_match(&c, date);
                    assignments.push(Assignment {
                        key: make_key(lemma, rows[ri].suffix),
                        forms: c.forms,
                    });
                    matched[ri] = true;
                }
            }

            // -- Phase B(a): drift re-match by etymology (forms edited but same etym) --
            // Only for rows whose FROZEN identity is the etym; a sig-frozen row that
            // merely got an etym enriched onto it must not be re-paired on etym alone
            // (an etymology renumber could otherwise hand it a different sense's forms).
            for ri in 0..original_len {
                if matched[ri] || rows[ri].status != Status::Active {
                    continue;
                }
                if !rows[ri].is_etym_frozen() {
                    continue;
                }
                let Some(etym) = rows[ri].etym else { continue };
                if let Some(ci) = remaining.iter().position(|c| c.etym == Some(etym)) {
                    let c = remaining.remove(ci);
                    notes.push(format!(
                        "{lemma}{} re-matched by etym:{etym} after form drift ({} -> {})",
                        suffix_note(rows[ri].suffix),
                        rows[ri].sig(),
                        c.sig()
                    ));
                    rows[ri].apply_match(&c, date);
                    assignments.push(Assignment {
                        key: make_key(lemma, rows[ri].suffix),
                        forms: c.forms,
                    });
                    matched[ri] = true;
                }
            }

            // -- Phase B(b): the unambiguous single-sense case --
            // Only reuse a vacated key for an unanchored sense when the lemma has
            // ALWAYS been single-sense: then a 1-out/1-in is almost certainly a
            // form correction, and the key has never meant anything else. For a
            // polysemous lemma we refuse to guess (that is exactly how an unrelated
            // sense would silently inherit a published key) and instead tombstone +
            // append, degrading the old key to the regular rule rather than to a
            // different irregular sense.
            let active_count = (0..original_len)
                .filter(|&ri| rows[ri].status == Status::Active)
                .count();
            let unmatched_active: Vec<usize> = (0..original_len)
                .filter(|&ri| !matched[ri] && rows[ri].status == Status::Active)
                .collect();
            if active_count == 1 && unmatched_active.len() == 1 && remaining.len() == 1 {
                let ri = unmatched_active[0];
                // Never reuse across a known etymology boundary.
                let etym_conflict = rows[ri].etym.is_some() && remaining[0].etym.is_some();
                if !etym_conflict {
                    let c = remaining.remove(0);
                    notes.push(format!(
                        "{lemma}{} re-matched as sole remaining sense after form drift ({} -> {})",
                        suffix_note(rows[ri].suffix),
                        rows[ri].sig(),
                        c.sig()
                    ));
                    rows[ri].apply_match(&c, date);
                    assignments.push(Assignment {
                        key: make_key(lemma, rows[ri].suffix),
                        forms: c.forms,
                    });
                    matched[ri] = true;
                }
            } else if !unmatched_active.is_empty() && !remaining.is_empty() {
                notes.push(format!(
                    "AMBIGUOUS: {lemma} [{}] has {} unmatched pinned sense(s) and {} unmatched dump sense(s) \
                     with no shared anchor; tombstoning the pinned one(s) and allocating new suffix(es). Review the lock diff.",
                    pos.as_str(),
                    unmatched_active.len(),
                    remaining.len()
                ));
            }

            // -- Phase C: allocate strictly-higher suffixes for new identities --
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
                let mut anchor = primary_anchor_token(&c, &uniq);
                if taken.contains(&anchor) {
                    anchor = format!("{anchor}#u{suffix}");
                }
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

            // -- Phase D: tombstone pinned senses that vanished from the dump --
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
        assignments
    }

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
                other => return Err(format!("invalid status {other:?} in lock row: {record:?}").into()),
            };
            let forms = split_forms(record.get(10).unwrap_or(""));
            let row = LockRow {
                lemma: record.get(0).unwrap_or("").to_string(),
                pos,
                suffix,
                anchor: record.get(3).unwrap_or("").to_string(),
                qid: opt(record.get(4)),
                sid: opt(record.get(5)),
                etym: record.get(6).and_then(|s| s.trim().parse::<u32>().ok()),
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

    /// Coverage histogram by anchor tier — answers "how much of the lock rests on
    /// strong vs. weak identity?" (research flagged qid/sid as sparse).
    pub fn coverage(&self) -> Coverage {
        let mut c = Coverage::default();
        for r in self.rows() {
            if r.status != Status::Active {
                c.tombstoned += 1;
                continue;
            }
            c.active += 1;
            if r.qid.is_some() {
                c.by_qid += 1;
            } else if r.sid.is_some() {
                c.by_sid += 1;
            } else if r.etym.is_some() {
                c.by_etym += 1;
            } else {
                c.by_sig += 1;
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

#[derive(Debug, Default)]
pub struct Coverage {
    pub active: usize,
    pub tombstoned: usize,
    pub by_qid: usize,
    pub by_sid: usize,
    pub by_etym: usize,
    pub by_sig: usize,
}

/// Compare a previous (committed) lock against a freshly-resolved one and return a
/// list of *violations*: ways in which an already-published key changed meaning.
/// Empty = safe to ship.
pub fn check_immutability(old: &Lock, new: &Lock) -> Vec<String> {
    let mut violations = Vec::new();

    // Index new rows by frozen identity and by (lemma,pos,suffix).
    let mut new_by_anchor: BTreeMap<(String, String, String), &LockRow> = BTreeMap::new();
    let mut new_by_suffix: BTreeMap<(String, String, u32), Vec<&LockRow>> = BTreeMap::new();
    let mut anchor_counts: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    for r in new.rows() {
        let akey = (r.lemma.clone(), r.pos.as_str().to_string(), r.anchor.clone());
        *anchor_counts.entry(akey.clone()).or_default() += 1;
        new_by_anchor.insert(akey, r);
        new_by_suffix
            .entry((r.lemma.clone(), r.pos.as_str().to_string(), r.suffix))
            .or_default()
            .push(r);
    }

    // Every frozen anchor must identify exactly one row within a (lemma,pos);
    // a duplicate means two keys share an identity (and the guard can't track them).
    for ((lemma, pos, anchor), n) in &anchor_counts {
        if *n > 1 {
            violations.push(format!(
                "{lemma} [{pos}] anchor `{anchor}` is shared by {n} rows (anchors must be unique per lemma)"
            ));
        }
    }

    for r in old.rows() {
        let id = (r.lemma.clone(), r.pos.as_str().to_string(), r.anchor.clone());
        match new_by_anchor.get(&id) {
            None => {
                if r.status == Status::Active {
                    violations.push(format!(
                        "{} [{}] anchor `{}` (suffix {}) was DROPPED; an active key must be tombstoned, not deleted",
                        r.lemma, r.pos.as_str(), r.anchor, r.suffix
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
            }
        }

        // A retired suffix must never be occupied by a *different* identity.
        if r.status == Status::Tombstone {
            let occupants = new_by_suffix.get(&(
                r.lemma.clone(),
                r.pos.as_str().to_string(),
                r.suffix,
            ));
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

/// Suffix 1 emits the bare lemma; >=2 appends the integer.
pub fn make_key(lemma: &str, suffix: u32) -> String {
    if suffix <= 1 {
        lemma.to_string()
    } else {
        format!("{lemma}{suffix}")
    }
}

fn suffix_note(suffix: u32) -> String {
    if suffix <= 1 {
        String::new()
    } else {
        suffix.to_string()
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

/// Anchor values (per tier) that are shared by more than one candidate in a batch.
/// A strong anchor (qid/sid/etym) is only safe to use bare when it is unique among
/// the lemma's candidates; otherwise the form signature must disambiguate it. This
/// matters most for nouns, where one entry can list several plural forms that all
/// inherit the entry's single qid/sid/etymology.
#[derive(Default)]
struct BatchUniq {
    qid_dup: std::collections::HashSet<String>,
    sid_dup: std::collections::HashSet<String>,
    etym_dup: std::collections::HashSet<u32>,
}

impl BatchUniq {
    fn of(candidates: &[Candidate]) -> Self {
        use std::collections::HashMap;
        let mut qid: HashMap<&str, u32> = HashMap::new();
        let mut sid: HashMap<&str, u32> = HashMap::new();
        let mut etym: HashMap<u32, u32> = HashMap::new();
        for c in candidates {
            if let Some(q) = &c.qid {
                *qid.entry(q).or_default() += 1;
            }
            if let Some(s) = &c.sid {
                *sid.entry(s).or_default() += 1;
            }
            if let Some(e) = c.etym {
                *etym.entry(e).or_default() += 1;
            }
        }
        BatchUniq {
            qid_dup: qid.into_iter().filter(|(_, n)| *n > 1).map(|(k, _)| k.to_string()).collect(),
            sid_dup: sid.into_iter().filter(|(_, n)| *n > 1).map(|(k, _)| k.to_string()).collect(),
            etym_dup: etym.into_iter().filter(|(_, n)| *n > 1).map(|(k, _)| k).collect(),
        }
    }

    fn qid_unique(&self, q: &str) -> bool {
        !self.qid_dup.contains(q)
    }
    fn sid_unique(&self, s: &str) -> bool {
        !self.sid_dup.contains(s)
    }
    fn etym_unique(&self, e: u32) -> bool {
        !self.etym_dup.contains(&e)
    }
}

/// A candidate matches a row when their strongest shared anchor agrees — and, when
/// that anchor value is shared by multiple candidates this batch, when the form
/// signatures also agree (so the right plural-variant pairs with the right row).
fn anchor_exact_match(row: &LockRow, c: &Candidate, uniq: &BatchUniq) -> bool {
    if let (Some(rq), Some(cq)) = (&row.qid, &c.qid)
        && rq == cq
        && (uniq.qid_unique(cq) || row.sig() == c.sig())
    {
        return true;
    }
    if let (Some(rs), Some(cs)) = (&row.sid, &c.sid)
        && rs == cs
        && (uniq.sid_unique(cs) || row.sig() == c.sig())
    {
        return true;
    }
    if let (Some(re), Some(ce)) = (row.etym, c.etym)
        && re == ce
        && (row.sig() == c.sig() || (row.is_etym_frozen() && uniq.etym_unique(ce)))
    {
        return true;
    }
    row.sig() == c.sig()
}

/// The frozen primary identity for a new row. Always unique within `(lemma, pos)`:
/// the strongest available anchor, plus a `#sig:` tiebreaker whenever that anchor
/// value is shared by another candidate (candidates are deduped by signature, so
/// the signature is always a unique disambiguator).
fn primary_anchor_token(c: &Candidate, uniq: &BatchUniq) -> String {
    if let Some(q) = &c.qid {
        return if uniq.qid_unique(q) {
            format!("qid:{q}")
        } else {
            format!("qid:{q}#sig:{}", c.sig())
        };
    }
    if let Some(s) = &c.sid {
        return if uniq.sid_unique(s) {
            format!("sid:{s}")
        } else {
            format!("sid:{s}#sig:{}", c.sig())
        };
    }
    if let Some(e) = c.etym {
        return if uniq.etym_unique(e) {
            format!("etym:{e}")
        } else {
            format!("etym:{e}#sig:{}", c.sig())
        };
    }
    format!("sig:{}", c.sig())
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

    fn keys(a: &[Assignment]) -> Vec<String> {
        let mut k: Vec<String> = a.iter().map(|x| x.key.clone()).collect();
        k.sort();
        k
    }

    #[test]
    fn cold_start_reproduces_legacy_lie() {
        // No regular-equal sense dropped -> start at suffix 1; alphabetical-by-forms order.
        let mut lock = Lock::new();
        let recline = cand(&["lies", "lay", "lying", "lain"]);
        let untruth = cand(&["lies", "lied", "lying", "lied"]);
        let a = lock.resolve("lie", Pos::Verb, vec![untruth, recline], false, "d1");
        let mut by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key.remove("lie").unwrap()[1], "lay"); // recline is the bare key
        assert_eq!(by_key.remove("lie2").unwrap()[1], "lied"); // untruth is lie2
    }

    #[test]
    fn cold_start_with_regular_starts_at_two() {
        // die: regular plural dropped (had_regular), single irregular -> die2, no bare die.
        let mut lock = Lock::new();
        let a = lock.resolve("die", Pos::Noun, vec![cand(&["dice"])], true, "d1");
        assert_eq!(keys(&a), vec!["die2".to_string()]);
    }

    #[test]
    fn reordered_forms_do_not_swap_keys() {
        // Establish lie/lie2, then a later dump returns the senses in a different
        // order AND with the untruth past-form edited. Keys must hold.
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
        );
        // Round-trip through CSV to ensure persistence carries identity.
        let dir = std::env::temp_dir().join("english_registry_test_swap");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("verb.lock.csv");
        lock.save(&path).unwrap();
        let mut lock2 = Lock::load(&path).unwrap();

        let a = lock2.resolve(
            "lie",
            Pos::Verb,
            vec![
                // recline reordered to second; untruth's forms identical (sig anchor holds)
                cand(&["lies", "lied", "lying", "lied"]),
                cand(&["lies", "lay", "lying", "lain"]),
            ],
            false,
            "d2",
        );
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["lie"][1], "lay", "recline must stay the bare key");
        assert_eq!(by_key["lie2"][1], "lied", "untruth must stay lie2");
    }

    #[test]
    fn etym_anchor_holds_through_form_edit() {
        // With an etymology anchor, editing a form must NOT renumber.
        let mut lock = Lock::new();
        let mut c1 = cand(&["lies", "lay", "lying", "lain"]);
        c1.etym = Some(1);
        let mut c2 = cand(&["lies", "lied", "lying", "lied"]);
        c2.etym = Some(2);
        lock.resolve("lie", Pos::Verb, vec![c1, c2], false, "d1");

        // d2: same etyms, but recline's past_part edited lain -> layed (a form drift)
        let mut e1 = cand(&["lies", "lay", "lying", "layed"]);
        e1.etym = Some(1);
        let mut e2 = cand(&["lies", "lied", "lying", "lied"]);
        e2.etym = Some(2);
        let a = lock.resolve("lie", Pos::Verb, vec![e2, e1], false, "d2");
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["lie"][3], "layed", "recline kept its key, forms updated");
        assert_eq!(by_key["lie2"][1], "lied");
    }

    #[test]
    fn new_sense_appends_and_existing_holds() {
        let mut lock = Lock::new();
        lock.resolve("run", Pos::Verb, vec![cand(&["runs", "ran", "running", "run"])], false, "d1");
        // d2: original sense unchanged + a brand-new homograph appears.
        let a = lock.resolve(
            "run",
            Pos::Verb,
            vec![
                cand(&["runs", "ran", "running", "run"]),
                cand(&["runs", "runned", "running", "runned"]),
            ],
            false,
            "d2",
        );
        assert_eq!(keys(&a), vec!["run".to_string(), "run2".to_string()]);
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["run"][1], "ran", "original sense keeps the bare key");
        assert_eq!(by_key["run2"][1], "runned", "new sense appended at run2");
    }

    #[test]
    fn vanished_sense_is_tombstoned_and_suffix_never_reused() {
        let mut lock = Lock::new();
        lock.resolve(
            "x",
            Pos::Noun,
            vec![cand(&["xen"]), cand(&["xes"])], // x -> xen, x2 -> xes
            false,
            "d1",
        );
        // d2: xen sense disappears; xes remains; a new sense appears.
        let a = lock.resolve("x", Pos::Noun, vec![cand(&["xes"]), cand(&["xim"])], false, "d2");
        // xen's suffix (1) is retired; xes keeps suffix 2; new sense gets 3 (not 1).
        assert_eq!(keys(&a), vec!["x2".to_string(), "x3".to_string()]);
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["x2"][0], "xes");
        assert_eq!(by_key["x3"][0], "xim");
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
        let a1 = lock.resolve("lie", Pos::Verb, mk(), false, "d1");
        let a2 = lock.resolve("lie", Pos::Verb, mk(), false, "d1");
        assert_eq!(keys(&a1), keys(&a2));
        assert_eq!(keys(&a1), vec!["lie".to_string(), "lie2".to_string()]);
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
        );
        // Hand-craft a "new" lock where the two anchors swapped suffixes.
        let mut new = Lock::new();
        for r in old.rows() {
            let mut nr = r.clone();
            nr.suffix = if r.suffix == 1 { 2 } else { 1 };
            new.insert_row(nr);
        }
        let v = check_immutability(&old, &new);
        assert!(!v.is_empty(), "swap must be flagged");
        assert!(v.iter().any(|m| m.contains("MOVED suffix")));
    }

    #[test]
    fn immutability_guard_allows_form_update_and_append() {
        let mut old = Lock::new();
        old.resolve("die", Pos::Noun, vec![cand(&["dice"])], true, "d1");
        let mut new = Lock::new();
        new.resolve("die", Pos::Noun, vec![cand(&["dice"])], true, "d1");
        // forms update in place + append a new sense: both legal.
        new.resolve("die", Pos::Noun, vec![cand(&["dice"]), cand(&["dies_alt"])], true, "d2");
        assert!(check_immutability(&old, &new).is_empty());
    }

    #[test]
    fn shared_strong_anchor_gets_sig_tiebreak() {
        // One noun entry with several plural forms shares its qid across candidates;
        // each emitted key must still get a UNIQUE anchor (qid + sig tiebreak), or
        // the immutability guard can't tell the rows apart.
        let mut lock = Lock::new();
        let mut c1 = cand(&["walrii"]);
        c1.qid = Some("Q40994".into());
        let mut c2 = cand(&["walrus"]);
        c2.qid = Some("Q40994".into());
        let mut c3 = cand(&["walrusses"]);
        c3.qid = Some("Q40994".into());
        lock.resolve("walrus", Pos::Noun, vec![c1, c2, c3], true, "d1");

        let anchors: Vec<String> = lock.rows().map(|r| r.anchor.clone()).collect();
        let distinct: std::collections::HashSet<_> = anchors.iter().collect();
        assert_eq!(anchors.len(), distinct.len(), "anchors must be unique: {anchors:?}");
        assert!(
            anchors.iter().all(|a| a.starts_with("qid:Q40994#sig:")),
            "{anchors:?}"
        );
        // A freshly-resolved lock must pass the guard against itself.
        assert!(check_immutability(&lock, &lock).is_empty());

        // And re-resolving the same dump is idempotent (keys hold).
        let mut c1 = cand(&["walrii"]);
        c1.qid = Some("Q40994".into());
        let mut c2 = cand(&["walrus"]);
        c2.qid = Some("Q40994".into());
        let mut c3 = cand(&["walrusses"]);
        c3.qid = Some("Q40994".into());
        let again = lock.resolve("walrus", Pos::Noun, vec![c1, c2, c3], true, "d2");
        assert_eq!(keys(&again), vec!["walrus2", "walrus3", "walrus4"]);
        assert!(check_immutability(&lock, &lock).is_empty());
    }

    #[test]
    fn co_etymology_uses_sig_tiebreak() {
        // Two senses share etym 2 -> etym not unique -> anchors carry sig tiebreak.
        let mut lock = Lock::new();
        let mut c1 = cand(&["dies"]);
        c1.etym = Some(2);
        let mut c2 = cand(&["dice"]);
        c2.etym = Some(2);
        lock.resolve("die", Pos::Noun, vec![c1, c2], false, "d1");
        let anchors: Vec<String> = lock.rows().map(|r| r.anchor.clone()).collect();
        assert!(anchors.iter().all(|a| a.starts_with("etym:2#sig:")), "{anchors:?}");
    }

    #[test]
    fn etym_renumber_does_not_swap_sig_frozen_rows() {
        // Two senses created sig-frozen (no etym), later ENRICHED with etyms. If a
        // future dump renumbers the etymology sections AND the forms drift, a
        // sig-frozen row must never grab the other sense's forms via the (renumbered)
        // etym — that is the silent run1<->run2 swap the lock exists to prevent.
        let mut lock = Lock::new();
        lock.resolve("x", Pos::Noun, vec![cand(&["alpha"]), cand(&["beta"])], false, "d1");
        // d2: same forms, now carrying etyms -> rows get etym enriched (anchors stay sig:).
        lock.resolve("x", Pos::Noun, vec![cand_e(&["alpha"], 1), cand_e(&["beta"], 2)], false, "d2");

        let dir = std::env::temp_dir().join("english_registry_test_etymswap");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("noun.lock.csv");
        lock.save(&path).unwrap();
        let before = Lock::load(&path).unwrap();

        // d3: etyms RENUMBERED (alpha now 2, beta now 1) AND both forms drift.
        lock.resolve("x", Pos::Noun, vec![cand_e(&["alpha2"], 2), cand_e(&["beta2"], 1)], false, "d3");

        let by_anchor: BTreeMap<String, &LockRow> =
            lock.rows().map(|r| (r.anchor.clone(), r)).collect();
        let a = by_anchor.get("sig:alpha").expect("alpha row");
        let b = by_anchor.get("sig:beta").expect("beta row");
        assert_eq!(a.suffix, 1);
        assert_eq!(b.suffix, 2);
        assert_eq!(a.forms, vec!["alpha".to_string()], "alpha must NOT inherit beta's forms");
        assert_eq!(b.forms, vec!["beta".to_string()], "beta must NOT inherit alpha's forms");
        // No published key silently changed meaning.
        let v = check_immutability(&before, &lock);
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn qid_enrichment_holds_key_through_drift() {
        // qid is durable, so once enriched it must absorb form drift even for a
        // row whose frozen anchor is a signature — in a polysemous lemma where the
        // signature alone would lose the match.
        let mut lock = Lock::new();
        lock.resolve("y", Pos::Noun, vec![cand(&["aaa"]), cand(&["bbb"])], false, "d1");
        lock.resolve("y", Pos::Noun, vec![cand_q(&["aaa"], "Q1"), cand_q(&["bbb"], "Q2")], false, "d2");
        // d3: the Q1 sense drifts aaa -> azz; the bare key must hold via qid.
        let asg = lock.resolve("y", Pos::Noun, vec![cand_q(&["azz"], "Q1"), cand_q(&["bbb"], "Q2")], false, "d3");
        assert_eq!(keys(&asg), vec!["y".to_string(), "y2".to_string()]);
        let by_key: BTreeMap<_, _> = asg.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["y"], vec!["azz".to_string()], "qid Q1 held the bare key through drift");
        assert_eq!(by_key["y2"], vec!["bbb".to_string()]);
    }

    #[test]
    fn ambiguous_drift_tombstones_and_appends() {
        // Two sig-frozen senses both drift with no shared anchor: the design must
        // tombstone the old suffixes and append at strictly-higher ones, never
        // letting a new sense inherit a published key.
        let mut lock = Lock::new();
        lock.resolve("z", Pos::Noun, vec![cand(&["p1"]), cand(&["q1"])], false, "d1");
        let dir = std::env::temp_dir().join("english_registry_test_ambig");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("noun.lock.csv");
        lock.save(&path).unwrap();
        let before = Lock::load(&path).unwrap();

        let asg = lock.resolve("z", Pos::Noun, vec![cand(&["p2"]), cand(&["q2"])], false, "d2");
        assert_eq!(keys(&asg), vec!["z3".to_string(), "z4".to_string()]);
        assert!(check_immutability(&before, &lock).is_empty());
        assert!(lock.notes.iter().any(|n| n.contains("AMBIGUOUS")));
    }

    #[test]
    fn reap_tombstones_absent_lemma() {
        let mut lock = Lock::new();
        lock.resolve("alpha", Pos::Noun, vec![cand(&["alphae"])], false, "d1");
        lock.resolve("beta", Pos::Noun, vec![cand(&["betae"])], false, "d1");
        // Next refresh resolves only "alpha"; "beta" vanished from the dump.
        let mut resolved = std::collections::HashSet::new();
        resolved.insert("alpha".to_string());
        let rep = lock.reap(Pos::Noun, &resolved, "d2", 0.90);
        assert_eq!(rep.tombstoned, 1);
        assert!(!rep.skipped);
        let beta = lock.rows().find(|r| r.lemma == "beta").unwrap();
        assert_eq!(beta.status, Status::Tombstone);
        let alpha = lock.rows().find(|r| r.lemma == "alpha").unwrap();
        assert_eq!(alpha.status, Status::Active);
        let emitted: Vec<String> = lock.emittable(Pos::Noun).into_iter().map(|(k, _)| k).collect();
        assert!(emitted.contains(&"alpha".to_string()));
        assert!(!emitted.contains(&"beta".to_string()), "reaped key must stop being emitted");
    }

    #[test]
    fn reap_skips_on_mass_absence() {
        // A partial/bad dump that would retire most of the lock is refused wholesale.
        let mut lock = Lock::new();
        lock.resolve("a", Pos::Noun, vec![cand(&["airr"])], false, "d1");
        lock.resolve("b", Pos::Noun, vec![cand(&["birr"])], false, "d1");
        lock.resolve("c", Pos::Noun, vec![cand(&["cirr"])], false, "d1");
        lock.resolve("d", Pos::Noun, vec![cand(&["dirr"])], false, "d1");
        let mut resolved = std::collections::HashSet::new();
        resolved.insert("a".to_string()); // 3 of 4 absent -> 75% > 10% guard
        let rep = lock.reap(Pos::Noun, &resolved, "d2", 0.10);
        assert!(rep.skipped);
        assert_eq!(rep.tombstoned, 0);
        assert!(lock.rows().all(|r| r.status == Status::Active), "nothing tombstoned on a bad dump");
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

    #[test]
    fn cold_start_orders_by_strongest_anchor_not_forms() {
        // etym 1 has lexicographically LARGER forms than etym 2. order_key must rank
        // by the strong anchor (etym) first, so etym 1 takes the bare key regardless
        // of spelling — the signature is the LAST resort, not the first.
        let mut lock = Lock::new();
        let mut e1 = cand(&["zzz"]);
        e1.etym = Some(1);
        let mut e2 = cand(&["aaa"]);
        e2.etym = Some(2);
        // pass in reverse order to prove ordering is by anchor, not input order
        let a = lock.resolve("w", Pos::Noun, vec![e2, e1], false, "d1");
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["w"], vec!["zzz".to_string()], "etym 1 takes the bare key despite larger forms");
        assert_eq!(by_key["w2"], vec!["aaa".to_string()]);
    }

    #[test]
    fn emit_collisions_detects_duplicate_keys() {
        let mut lock = Lock::new();
        // suffix 1 and suffix 0 both map to make_key == bare "w".
        lock.insert_row(row("w", 1, "sig:a", None, &["a"]));
        lock.insert_row(row("w", 0, "sig:b", None, &["b"]));
        let c = lock.emit_collisions(Pos::Noun);
        assert_eq!(c.len(), 1, "{c:?}");
        assert!(c[0].contains("emit key `w`"), "{c:?}");
    }

    #[test]
    fn validate_flags_suffix_zero_and_anchor_tier_mismatch() {
        let mut lock = Lock::new();
        lock.insert_row(row("z", 0, "sig:z", None, &["z"])); // suffix 0
        lock.insert_row(row("q", 2, "qid:Q1", None, &["q"])); // qid-tier anchor, qid field empty
        let v = lock.validate();
        assert!(v.iter().any(|m| m.contains("suffix 0")), "{v:?}");
        assert!(
            v.iter().any(|m| m.contains("qid-tier but the qid field is empty")),
            "{v:?}"
        );
        // A clean lock validates empty.
        let mut ok = Lock::new();
        ok.insert_row(row("good", 2, "qid:Q9", Some("Q9"), &["goods"]));
        assert!(ok.validate().is_empty(), "{:?}", ok.validate());
    }

    #[test]
    fn tombstone_reappear_gets_unique_anchor() {
        // A sense that vanished (tombstoned) then reappears re-derives the same anchor;
        // the new row must get a UNIQUE anchor (not collide with its own tombstone) and
        // a strictly-higher suffix. Named guard for the bug the proptest first caught.
        let mut lock = Lock::new();
        let mut c = cand(&["alpha"]);
        c.etym = Some(2);
        lock.resolve("x", Pos::Noun, vec![c], false, "d1"); // x -> etym:2 @1
        lock.resolve("x", Pos::Noun, vec![], false, "d2"); // vanishes -> tombstone @1
        let mut c2 = cand(&["alpha"]);
        c2.etym = Some(2);
        let a = lock.resolve("x", Pos::Noun, vec![c2], false, "d3"); // reappears

        assert_eq!(keys(&a), vec!["x2".to_string()], "reappearing sense gets a new suffix");
        let anchors: Vec<String> = lock.rows().map(|r| r.anchor.clone()).collect();
        let distinct: std::collections::HashSet<_> = anchors.iter().collect();
        assert_eq!(anchors.len(), distinct.len(), "anchors must stay unique: {anchors:?}");
        assert!(check_immutability(&lock, &lock).is_empty());
    }

    #[test]
    fn reap_respects_max_fraction_boundary() {
        let build = || {
            let mut lock = Lock::new();
            lock.resolve("a", Pos::Noun, vec![cand(&["air"])], false, "d1");
            lock.resolve("b", Pos::Noun, vec![cand(&["bir"])], false, "d1");
            lock.resolve("c", Pos::Noun, vec![cand(&["cir"])], false, "d1");
            lock.resolve("d", Pos::Noun, vec![cand(&["dir"])], false, "d1");
            lock
        };
        let mut resolved = std::collections::HashSet::new();
        resolved.insert("a".to_string());
        resolved.insert("b".to_string());
        resolved.insert("c".to_string());

        // 1 of 4 absent = 25%; with max_fraction 0.25 that is NOT > 0.25 -> reaps.
        let mut at = build();
        let rep = at.reap(Pos::Noun, &resolved, "d2", 0.25);
        assert!(!rep.skipped, "25% at the 0.25 boundary should reap");
        assert_eq!(rep.tombstoned, 1);

        // 1 of 4 absent vs a 0.10 ceiling = 25% > 10% -> skipped.
        let mut over = build();
        let rep = over.reap(Pos::Noun, &resolved, "d2", 0.10);
        assert!(rep.skipped, "25% over the 0.10 ceiling must be refused");
        assert_eq!(rep.tombstoned, 0);
    }

    #[test]
    fn etym_frozen_row_rematches_on_form_drift() {
        // Complement to the sig-frozen no-swap test: an etym-FROZEN row whose forms
        // drift (same etym) DOES carry its key forward, rather than tombstoning.
        let mut lock = Lock::new();
        let mut c1 = cand(&["aaa"]);
        c1.etym = Some(1);
        let mut c2 = cand(&["bbb"]);
        c2.etym = Some(2);
        lock.resolve("x", Pos::Noun, vec![c1, c2], false, "d1"); // etym:1 -> x, etym:2 -> x2

        let mut e1 = cand(&["azz"]); // etym 1's form drifts aaa -> azz
        e1.etym = Some(1);
        let mut e2 = cand(&["bbb"]);
        e2.etym = Some(2);
        let a = lock.resolve("x", Pos::Noun, vec![e1, e2], false, "d2");
        let by_key: BTreeMap<_, _> = a.iter().map(|x| (x.key.clone(), x.forms.clone())).collect();
        assert_eq!(by_key["x"], vec!["azz".to_string()], "etym-frozen row kept its key through drift");
        assert_eq!(by_key["x2"], vec!["bbb".to_string()]);
        assert!(check_immutability(&lock, &lock).is_empty());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// One observed sense: forms + the full set of anchor tiers (etym / qid / sid),
    /// so the property test exercises qid/sid matching and the `#sig` tiebreaks, not
    /// just sig/etym. Small alphabets keep the search dense (and force collisions).
    type Sense = (Vec<String>, Option<u32>, Option<String>, Option<String>);
    fn sense() -> impl Strategy<Value = Sense> {
        (
            prop::collection::vec("[a-c]{1,3}", 1..3usize),
            prop::option::of(0u32..3),
            prop::option::of("Q[1-3]"),
            prop::option::of("s[1-3]"),
        )
    }

    /// A stream of dumps. Each dump carries an independent `had_regular` flag (to
    /// exercise the cold-start-at-2 branch) and senses for TWO lemmas (to confirm the
    /// per-(lemma,pos) buckets stay independent).
    fn dumps() -> impl Strategy<Value = Vec<(bool, Vec<Sense>, Vec<Sense>)>> {
        let dump = (
            any::<bool>(),
            prop::collection::vec(sense(), 0..3usize),
            prop::collection::vec(sense(), 0..3usize),
        );
        prop::collection::vec(dump, 1..6usize)
    }

    fn to_candidates(senses: &[Sense]) -> Vec<Candidate> {
        // Dedup by signature, as the extract layer does before calling resolve.
        let mut by_sig: BTreeMap<String, Candidate> = BTreeMap::new();
        for (forms, etym, qid, sid) in senses {
            let mut c = Candidate::new(forms.clone());
            c.etym = *etym;
            c.qid = qid.clone();
            c.sid = sid.clone();
            by_sig.entry(c.sig()).or_insert(c);
        }
        by_sig.into_values().collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1024))]

        /// Across an arbitrary stream of refreshes — over qid/sid/etym/sig anchors,
        /// both had_regular branches, and two independent lemmas — no published
        /// (active) anchor ever changes its suffix and no tombstoned suffix is
        /// revived/reused. Verified by check_immutability between consecutive on-disk
        /// states and by an independent (lemma, anchor) -> suffix monotonicity ledger.
        /// Exercises the real resolve -> save -> load cycle.
        #[test]
        fn published_keys_never_move_or_revive(dumps in dumps()) {
            let dir = std::env::temp_dir().join(format!("english_proptest_lock_{}", std::process::id()));
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join("noun.lock.csv");

            let mut lock = Lock::new();
            let mut seen: std::collections::HashMap<(String, String), u32> = Default::default();

            for (i, (had_regular, sa, sb)) in dumps.iter().enumerate() {
                for (lemma, senses) in [("w", sa), ("x", sb)] {
                    let candidates = to_candidates(senses);

                    lock.save(&path).unwrap();
                    let before = Lock::load(&path).unwrap();
                    lock.resolve(lemma, Pos::Noun, candidates, *had_regular, &format!("d{i}"));

                    // No published key changed meaning between the two on-disk states.
                    prop_assert!(check_immutability(&before, &lock).is_empty());
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
