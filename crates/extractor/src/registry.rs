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
    fn parse(s: &str) -> Status {
        match s {
            "tombstone" => Status::Tombstone,
            _ => Status::Active,
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
    /// Deterministic sort key for ordering brand-new identities.
    fn order_key(&self) -> (String, String, String, u32) {
        (
            self.sig(),
            self.qid.clone().unwrap_or_default(),
            self.sid.clone().unwrap_or_default(),
            self.etym.unwrap_or(0),
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
            for ri in 0..original_len {
                if matched[ri] || rows[ri].status != Status::Active {
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
            remaining.sort_by_key(|c| c.order_key());
            for (offset, c) in remaining.into_iter().enumerate() {
                let suffix = base + offset as u32;
                let anchor = primary_anchor_token(&c, &uniq);
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
            let row = LockRow {
                lemma: record.get(0).unwrap_or("").to_string(),
                pos,
                suffix: record.get(2).unwrap_or("0").trim().parse().unwrap_or(0),
                anchor: record.get(3).unwrap_or("").to_string(),
                qid: opt(record.get(4)),
                sid: opt(record.get(5)),
                etym: record.get(6).and_then(|s| s.trim().parse::<u32>().ok()),
                status: Status::parse(record.get(7).unwrap_or("active").trim()),
                first_seen: record.get(8).unwrap_or("").to_string(),
                last_seen: record.get(9).unwrap_or("").to_string(),
                forms: split_forms(record.get(10).unwrap_or("")),
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
        && (uniq.etym_unique(ce) || row.sig() == c.sig())
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
}
