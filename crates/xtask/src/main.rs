//! Workspace task runner (`cargo xtask <command>`), and the home of the
//! project's stability machinery:
//!
//! - `refresh-data` — full dump-driven regeneration (delegates to the extractor
//!   binary in release mode);
//! - `check-registry` — THE CI gate: internal lock consistency, cross-version
//!   key immutability against the PR's merge-base, a dump-free byte comparison
//!   of the committed tables against what the lockfiles regenerate, an
//!   emitter-independent lock↔table membership check, and override validation;
//! - `regen-tables` — dump-free table emission from the lockfiles (use when
//!   only the emission format changed, not the data);
//! - `compact` — the sanctioned data-major cruft collection: drops tombstones,
//!   closes suffix gaps, bumps DATA_MAJOR (which is what makes check-registry
//!   accept the renumbering);
//! - `migrate-anchors` — one-time re-anchoring of legacy qid/etym rows to the
//!   exact-match sid/sig tiers (bumps DATA_MAJOR);
//! - `accuracy` — dump-driven measurement of rules+tables together;
//! - `seed-assignments` / `report-coverage` — one-time bootstrap and anchor
//!   telemetry.
//!
//! Stability model in one paragraph: published `_<n>` sense keys never change
//! meaning within a data-major version — check-registry fails any PR that
//! moves/drops/revives one. A deliberate re-key ships as a DATA_MAJOR bump
//! (visible in the diff, waives cross-version checks for exactly that PR) and
//! a semver-major crate release. ENGLISH_ALLOW_RELOCK exists as a local
//! development escape hatch only and is never set in CI.

use extractor::bootstrap::{generate_tables, load_locks, parse_phf_pairs, save_locks, seed};
use extractor::helpers::Pos;
use extractor::registry::{CompactReport, Lock, Overrides, check_immutability, make_key};
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

struct RefreshDataArgs {
    dump_path: PathBuf,
    generated_dir: Option<PathBuf>,
    artifacts_dir: Option<PathBuf>,
    assignments_dir: Option<PathBuf>,
    data_date: Option<String>,
    with_checks: bool,
    allow_partial_refresh: bool,
}

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

fn default_generated_dir() -> Result<PathBuf, Box<dyn Error>> {
    Ok(workspace_root()?.join("crates/english/generated"))
}

fn default_assignments_dir() -> Result<PathBuf, Box<dyn Error>> {
    Ok(workspace_root()?.join("data/assignments"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("refresh-data") => refresh_data(args.collect()),
        Some("seed-assignments") => seed_assignments(args.collect()),
        Some("check-registry") => check_registry(),
        Some("accuracy") => accuracy(args.collect()),
        Some("compact") => compact(),
        Some("migrate-anchors") => migrate_anchors(),
        Some("regen-tables") => regen_tables(),
        Some("report-coverage") => report_coverage(),
        Some("-h") | Some("--help") | None => {
            print_usage();
            Ok(())
        }
        Some(command) => Err(format!("unknown xtask command: {command}").into()),
    }
}

// --------------------------------------------------------------------------
// seed-assignments: one-time bootstrap of the lockfiles from committed tables.
// --------------------------------------------------------------------------
fn seed_assignments(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut generated_dir = default_generated_dir()?;
    let mut assignments_dir = default_assignments_dir()?;
    // The data the tables were built from (README: 2025-08-17).
    let mut data_date = "2025-08-17".to_string();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--generated-dir" => generated_dir = PathBuf::from(req(&mut iter, "--generated-dir")?),
            "--assignments-dir" => {
                assignments_dir = PathBuf::from(req(&mut iter, "--assignments-dir")?)
            }
            "--data-date" => data_date = req(&mut iter, "--data-date")?,
            "-h" | "--help" => {
                eprintln!(
                    "Usage: cargo xtask seed-assignments [--data-date YYYY-MM-DD] [--generated-dir DIR] [--assignments-dir DIR]"
                );
                return Ok(());
            }
            other => return Err(format!("unknown flag for `seed-assignments`: {other}").into()),
        }
    }

    seed(&generated_dir, &assignments_dir, &data_date)?;
    println!(
        "Seeded assignment lockfiles. Run `git diff {}` — the generated tables should be unchanged.",
        generated_dir.display()
    );
    Ok(())
}

// --------------------------------------------------------------------------
// check-registry: hard CI gate. Fails if any previously-committed key changed
// meaning, or if the working lock is internally inconsistent.
// --------------------------------------------------------------------------
fn check_registry() -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let assignments_dir = default_assignments_dir()?;
    let generated_dir = default_generated_dir()?;
    let (noun, verb, adj) = load_locks(&assignments_dir)?;
    // The reviewed rematch sanctions: consulted so a sanctioned re-anchor is not
    // flagged as a drop, and cross-checked so the file itself can never go stale.
    let overrides = Overrides::load(assignments_dir.join("overrides.csv"))?;

    let mut violations = Vec::new();

    // Escape hatch for a *deliberate* re-key: ENGLISH_ALLOW_RELOCK skips the
    // cross-version immutability comparison. It is never set in CI — using it there
    // requires a visible, reviewable edit to .github/workflows/ci.yml. Internal
    // consistency and lock<->table sync are ALWAYS enforced.
    let allow_relock = env::var("ENGLISH_ALLOW_RELOCK")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);

    // Baseline = the commit this branch diverged from on the target branch, so we
    // gate the change THIS branch/PR makes to the lockfiles. Comparing against HEAD
    // is useless under CI (the checked-out working tree IS HEAD), so a key swap that
    // arrives inside a PR would slip through. We use the merge-base with the PR base
    // ref (GITHUB_BASE_REF on GitHub, else origin/main), which needs full history
    // (actions/checkout with fetch-depth: 0).
    let mut baseline = if allow_relock {
        None
    } else {
        resolve_baseline(&root)?
    };
    if allow_relock {
        println!(
            "check-registry: ENGLISH_ALLOW_RELOCK set — cross-version immutability SKIPPED \
             (local relock escape hatch); internal consistency + table sync still enforced."
        );
    } else {
        match &baseline {
            Some(rev) => println!("check-registry: gating lock changes against baseline {rev}"),
            None => println!(
                "note: no baseline ref resolved (origin/main / GITHUB_BASE_REF); running internal + sync checks only."
            ),
        }
    }

    // Stability policy: published keys are immutable WITHIN a data-major version.
    // A PR that bumps data/assignments/DATA_MAJOR is a sanctioned compaction /
    // re-key release (`cargo xtask compact`): cross-version immutability is waived
    // for exactly that PR, loudly. The bump itself is what reviewers sign off on —
    // it is the versioned, visible replacement for silently mutating keys.
    let working_major = read_data_major_file(&assignments_dir.join("DATA_MAJOR"))?;
    if let Some(rev) = &baseline {
        let base_major = match git_show(&root, &format!("{rev}:data/assignments/DATA_MAJOR"))? {
            Some(s) => parse_data_major(&s)
                .ok_or_else(|| format!("unparseable DATA_MAJOR at baseline {rev}: {s:?}"))?,
            None => 1, // file predates the policy -> data-major 1
        };
        if working_major > base_major {
            let working_version = read_crate_version(&root.join("crates/english/Cargo.toml"))?;
            let base_version = git_show(&root, &format!("{rev}:crates/english/Cargo.toml"))?
                .and_then(|s| parse_crate_version(&s));
            match base_version {
                Some(base_version)
                    if is_breaking_version_bump(base_version, working_version) => {}
                Some(base_version) => violations.push(format!(
                    "DATA_MAJOR bumped {base_major} -> {working_major}, but english crate version did not make a semver-breaking bump ({} -> {}). A data re-key must ship as a breaking crate release.",
                    base_version,
                    working_version
                )),
                None => violations.push(format!(
                    "DATA_MAJOR bumped {base_major} -> {working_major}, but the baseline english crate version could not be read; refusing to waive a re-key without release-version evidence"
                )),
            }
            println!(
                "check-registry: DATA_MAJOR bump {base_major} -> {working_major}: SANCTIONED \
                 compaction/re-key release — cross-version immutability waived for this change \
                 only when the english crate version also makes a breaking bump; internal \
                 consistency + table sync still enforced."
            );
            baseline = None;
        } else if working_major < base_major {
            violations.push(format!(
                "DATA_MAJOR went backwards ({base_major} -> {working_major}); the data-major version is append-only"
            ));
        }
    }

    // Anchor-tier coverage on every gate run: most rows are content-anchored
    // (`sig:`) and only `sid:` rows survive form drift without a review — keep
    // the mix visible in CI logs so erosion or stagnation is noticed, not
    // discovered after a bad refresh. legacy>0 means a pre-migration anchor
    // slipped in (validate() also rejects it).
    for (name, lock) in [("noun", &noun), ("verb", &verb), ("adj", &adj)] {
        let c = lock.coverage();
        println!(
            "coverage {name:>4}: active={} (sid={} sig={} legacy={})  tombstoned={}",
            c.active, c.by_sid, c.by_sig, c.legacy, c.tombstoned
        );
    }

    for (pos, working) in [(Pos::Noun, &noun), (Pos::Verb, &verb), (Pos::Adj, &adj)] {
        // 1. Internal consistency (unique anchors, no two identities share a suffix)
        //    plus strict structural validation (suffix>=1, anchor<->fields, no emit
        //    key collisions). Always enforced, never relaxed by the relock window.
        violations.extend(check_immutability(working, working, &overrides));
        violations.extend(working.validate());
        // Overrides must always correspond to real rows (pending or applied);
        // a stale override is a lie about what is sanctioned.
        violations.extend(overrides.check(working, pos, false));

        // 2. Immutability vs. the baseline lock, if present there.
        if let Some(rev) = &baseline {
            let rel = format!("data/assignments/{}.lock.csv", pos.as_str());
            match git_show(&root, &format!("{rev}:{rel}"))? {
                Some(csv) => {
                    let base_lock = load_lock_from_str(&csv)?;
                    violations.extend(check_immutability(&base_lock, working, &overrides));
                }
                None => {
                    println!(
                        "note: {rel} absent at {rev} (new file); skipping cross-version check for {}.",
                        pos.as_str()
                    );
                }
            }
        }
    }

    // 3. Lock <-> table sync (dump-free): the committed PHF tables must be exactly
    //    what the lockfiles regenerate. Catches a hand-edited lock or table, or a
    //    forgotten regeneration, that would ship inflections the lock doesn't back.
    //    Use a process-unique temp dir so concurrent/stale runs can't cross-contaminate.
    let tmp = env::temp_dir().join(format!("english_xtask_sync_{}", process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;
    generate_tables(&noun, &verb, &adj, &tmp)?;
    for name in ["noun_phf.rs", "verb_phf.rs", "adj_phf.rs"] {
        // A missing committed table is a violation, not "in sync" — never treat an
        // absent file as equal to the regenerated one.
        let committed = match fs::read(generated_dir.join(name)) {
            Ok(bytes) => bytes,
            Err(_) => {
                violations.push(format!(
                    "{name} is missing from {} (expected a committed PHF table)",
                    generated_dir.display()
                ));
                continue;
            }
        };
        let regenerated = fs::read(tmp.join(name))?; // just written by generate_tables
        if committed != regenerated {
            violations.push(format!(
                "{name} is out of sync with the lockfiles (run `cargo xtask refresh-data` or reconcile the lock)"
            ));
        }
    }
    let _ = fs::remove_dir_all(&tmp);

    // 4. Emitter-independent membership: parse the committed tables themselves and
    //    require a 1:1 correspondence between active lock rows and table keys. The
    //    byte-compare above regenerates with the SAME emitter it verifies, so a bug
    //    in the emitter (e.g. a filter silently dropping rows) byte-matches its own
    //    output; this check reads the tables through an independent path
    //    (parse_phf_pairs) and catches exactly that class.
    for (pos, lock, name) in [
        (Pos::Noun, &noun, "noun_phf.rs"),
        (Pos::Verb, &verb, "verb_phf.rs"),
        (Pos::Adj, &adj, "adj_phf.rs"),
    ] {
        let table_path = generated_dir.join(name);
        if !table_path.exists() {
            continue; // already flagged as missing above
        }
        let table_keys: std::collections::HashSet<String> = parse_phf_pairs(&table_path)?
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        let mut active = 0usize;
        let mut missing: Vec<String> = Vec::new();
        for r in lock.rows() {
            if r.pos != pos || r.status != extractor::registry::Status::Active {
                continue;
            }
            active += 1;
            let key = make_key(&r.lemma, r.suffix);
            if !table_keys.contains(&key) {
                missing.push(key);
            }
        }
        if !missing.is_empty() {
            let shown = missing
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            violations.push(format!(
                "{name}: {} active lock row(s) have no table entry (e.g. {shown}{})",
                missing.len(),
                if missing.len() > 10 { ", ..." } else { "" }
            ));
        }
        if table_keys.len() != active {
            violations.push(format!(
                "{name} has {} keys but the {} lock has {} active rows — table and lock disagree",
                table_keys.len(),
                pos.as_str(),
                active
            ));
        }
    }

    if violations.is_empty() {
        println!("check-registry: OK — no published key changed meaning; tables match the lock.");
        Ok(())
    } else {
        eprintln!(
            "check-registry: FAILED — {} violation(s):",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {v}");
        }
        process::exit(1);
    }
}

/// The baseline revision to gate lock changes against: the merge-base of `HEAD` with
/// the target branch (`GITHUB_BASE_REF` on a GitHub PR, otherwise `origin/main`).
/// Using the merge-base (the point this branch diverged) means new keys added on the
/// base branch after divergence are not falsely flagged as dropped. Returns `None`
/// when no such ref exists (e.g. a fresh repo with no remote), in which case only the
/// internal-consistency and sync checks run.
fn resolve_baseline(root: &Path) -> Result<Option<String>, Box<dyn Error>> {
    // When a PR base ref is given, gate against EXACTLY that branch — never silently
    // fall back to main (which would compare against the wrong baseline). Without one
    // (local runs / pushes), main is the natural baseline.
    let base_ref = env::var("GITHUB_BASE_REF")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let in_ci = base_ref.is_some() || env::var("CI").is_ok();
    let candidates: Vec<String> = match &base_ref {
        Some(b) => vec![format!("origin/{b}"), b.clone()],
        None => vec!["origin/main".to_string(), "main".to_string()],
    };

    let head = git_rev_parse(root, "HEAD")?;
    for base in &candidates {
        if let Some(mb) = git_merge_base(root, base, "HEAD")? {
            // On a push-to-main CI run there is no GITHUB_BASE_REF and origin/main
            // IS the pushed commit, so the merge-base equals HEAD and the lock
            // would be diffed against itself — a vacuous gate (comparing against
            // HEAD is exactly what the baseline exists to avoid). Gate against
            // the previous tip (first parent) instead so direct pushes to main
            // are still checked.
            if in_ci && Some(&mb) == head.as_ref() {
                // Prefer the true pre-push remote tip (github.event.before,
                // exported by ci.yml as GITHUB_EVENT_BEFORE): HEAD^ only gates
                // the FINAL commit of a multi-commit direct push. All-zeros
                // means branch creation; an unresolvable sha (force push that
                // discarded it) falls back to HEAD^.
                if let Some(before) = env::var("GITHUB_EVENT_BEFORE")
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && !s.chars().all(|c| c == '0'))
                    && let Some(sha) = git_rev_parse(root, &before)?
                {
                    return Ok(Some(sha));
                }
                return match git_rev_parse(root, "HEAD^")? {
                    Some(prev) => Ok(Some(prev)),
                    None => Err(
                        "check-registry: the baseline resolved to HEAD itself and HEAD has no \
                         parent; refusing to run a vacuous cross-version check in CI."
                            .into(),
                    ),
                };
            }
            return Ok(Some(mb));
        }
    }

    // No baseline resolved. Under CI this is fail-open and must NOT pass: a shallow
    // checkout, a missing base fetch, or a renamed default branch would otherwise
    // silently disable the headline cross-version immutability check while CI stays
    // green. Locally (no CI) it's fine to skip the cross-version comparison.
    if in_ci {
        return Err(format!(
            "check-registry: could not resolve a baseline ref ({}) to gate against. \
             Ensure full history (actions/checkout fetch-depth: 0) and that the base branch is \
             fetched. Refusing to pass with the cross-version immutability check disabled.",
            candidates.join(" or ")
        )
        .into());
    }
    Ok(None)
}

/// `git merge-base <base> HEAD`; returns `None` if `base` is unknown locally.
fn git_merge_base(root: &Path, base: &str, head: &str) -> Result<Option<String>, Box<dyn Error>> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["merge-base", base, head])
        .output()?;
    if out.status.success() {
        let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok((!sha.is_empty()).then_some(sha))
    } else {
        Ok(None)
    }
}

// --------------------------------------------------------------------------
// accuracy: measure "% of plain attested Wiktionary forms the library
// reproduces" per POS, against the CURRENT committed tables + rule engine.
// Defaults to the cached filtered dataset so it doesn't need the raw 20 GB dump.
// --------------------------------------------------------------------------
fn accuracy(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let mut dump = root.join("data/intermediate/english_filtered.jsonl");

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dump" => dump = PathBuf::from(req(&mut iter, "--dump")?),
            "-h" | "--help" => {
                eprintln!("Usage: cargo xtask accuracy [--dump /path/to/rawwiki.jsonl]");
                eprintln!("Defaults to the cached data/intermediate/english_filtered.jsonl.");
                return Ok(());
            }
            other => return Err(format!("unknown flag for `accuracy`: {other}").into()),
        }
    }
    if !dump.exists() {
        return Err(format!(
            "no dump at {} — pass one with `--dump` (the raw Wiktextract JSONL), or run \
             `cargo xtask refresh-data` once to populate the cached filtered dataset.",
            dump.display()
        )
        .into());
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .current_dir(&root)
        .args([
            "run",
            "-p",
            "extractor",
            "--release",
            "--features",
            "checks",
            "--",
        ])
        .arg("--dump")
        .arg(&dump)
        .arg("--checks-only")
        .status()?;
    if status.success() {
        Ok(())
    } else {
        process::exit(status.code().unwrap_or(1));
    }
}

// --------------------------------------------------------------------------
// compact: the deliberate cruft-collection for a data-major release. Deletes
// tombstones, closes suffix gaps, regenerates the tables, and bumps DATA_MAJOR
// so check-registry treats the renumbering as sanctioned.
// --------------------------------------------------------------------------
fn compact() -> Result<(), Box<dyn Error>> {
    let assignments_dir = default_assignments_dir()?;
    let generated_dir = default_generated_dir()?;
    let major_path = assignments_dir.join("DATA_MAJOR");
    let (mut noun, mut verb, mut adj) = load_locks(&assignments_dir)?;

    let mut total = CompactReport::default();
    for (name, lock) in [("noun", &mut noun), ("verb", &mut verb), ("adj", &mut adj)] {
        let r = lock.compact();
        println!(
            "compact {name:>4}: dropped {} tombstone(s), renumbered {} key(s)",
            r.tombstones_dropped, r.renumbered
        );
        total.tombstones_dropped += r.tombstones_dropped;
        total.renumbered += r.renumbered;
    }

    if total.is_noop() {
        println!("compact: nothing to do — no tombstones, no suffix gaps. DATA_MAJOR unchanged.");
        return Ok(());
    }

    // A compacted lock must still be structurally valid before anything is written.
    let invalid: Vec<String> = noun
        .validate()
        .into_iter()
        .chain(verb.validate())
        .chain(adj.validate())
        .collect();
    if !invalid.is_empty() {
        return Err(format!(
            "refusing to write compacted lockfiles — {} structural violation(s):\n  - {}",
            invalid.len(),
            invalid.join("\n  - ")
        )
        .into());
    }

    save_locks(&assignments_dir, &noun, &verb, &adj)?;
    generate_tables(&noun, &verb, &adj, &generated_dir)?;

    let old_major = read_data_major_file(&major_path)?;
    let new_major = old_major + 1;
    fs::write(&major_path, format!("{new_major}\n"))?;

    println!(
        "compact: dropped {} tombstone(s), renumbered {} published key(s); DATA_MAJOR {old_major} -> {new_major}.",
        total.tombstones_dropped, total.renumbered
    );
    println!(
        "This is a BREAKING data change: ship it as a semver-major release and call out the \
         renumbered keys in the changelog. check-registry accepts it because of the DATA_MAJOR bump."
    );
    Ok(())
}

// --------------------------------------------------------------------------
// migrate-anchors: one-time re-anchoring to the exact-match model. Rewrites
// every legacy qid/etym-tier frozen anchor to the row's current form signature
// (sid/sig rows are untouched); suffixes, forms, and statuses never change, so
// the emitted tables are byte-identical. Rewriting frozen anchors is exactly
// what check-registry flags, so this bumps DATA_MAJOR like compact does.
// --------------------------------------------------------------------------
fn migrate_anchors() -> Result<(), Box<dyn Error>> {
    let assignments_dir = default_assignments_dir()?;
    let generated_dir = default_generated_dir()?;
    let major_path = assignments_dir.join("DATA_MAJOR");
    let (mut noun, mut verb, mut adj) = load_locks(&assignments_dir)?;

    let mut total = 0usize;
    for (name, lock) in [("noun", &mut noun), ("verb", &mut verb), ("adj", &mut adj)] {
        let n = lock.migrate_to_exact_anchors();
        println!("migrate-anchors {name:>4}: re-anchored {n} row(s)");
        total += n;
    }
    if total == 0 {
        println!("migrate-anchors: nothing to do — every anchor is already sid/sig tier.");
        return Ok(());
    }

    let invalid: Vec<String> = noun
        .validate()
        .into_iter()
        .chain(verb.validate())
        .chain(adj.validate())
        .collect();
    if !invalid.is_empty() {
        return Err(format!(
            "refusing to write migrated lockfiles — {} structural violation(s):\n  - {}",
            invalid.len(),
            invalid.join("\n  - ")
        )
        .into());
    }

    save_locks(&assignments_dir, &noun, &verb, &adj)?;
    // Anchors are not part of the emitted tables; regenerate anyway to prove it.
    generate_tables(&noun, &verb, &adj, &generated_dir)?;

    let old_major = read_data_major_file(&major_path)?;
    let new_major = old_major + 1;
    fs::write(&major_path, format!("{new_major}\n"))?;
    println!(
        "migrate-anchors: re-anchored {total} row(s); DATA_MAJOR {old_major} -> {new_major}. \
         Published keys and emitted forms are unchanged; the bump sanctions the anchor rewrite."
    );
    Ok(())
}

/// Read `data/assignments/DATA_MAJOR`; a missing file means data-major 1 (the
/// policy's epoch). An unparseable file is a hard error, never a silent default.
fn read_data_major_file(path: &Path) -> Result<u32, Box<dyn Error>> {
    if !path.exists() {
        return Ok(1);
    }
    let s = fs::read_to_string(path)?;
    parse_data_major(&s)
        .ok_or_else(|| format!("unparseable DATA_MAJOR file {path:?}: {s:?}").into())
}

fn parse_data_major(s: &str) -> Option<u32> {
    s.trim().parse::<u32>().ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CrateVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl std::fmt::Display for CrateVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Parse the package version from a Cargo.toml string. This tiny parser is
/// intentionally dependency-free for xtask: it reads the first `version =` line,
/// which in `crates/english/Cargo.toml` belongs to `[package]`.
fn parse_crate_version(toml: &str) -> Option<CrateVersion> {
    let version = toml.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        let value = value
            .split('#')
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"');
        (key.trim() == "version").then_some(value)
    })?;
    let mut parts = version.split('.');
    Some(CrateVersion {
        major: parts.next()?.parse().ok()?,
        minor: parts.next()?.parse().ok()?,
        patch: parts.next()?.parse().ok()?,
    })
}

fn read_crate_version(path: &Path) -> Result<CrateVersion, Box<dyn Error>> {
    let s = fs::read_to_string(path)?;
    parse_crate_version(&s).ok_or_else(|| format!("unparseable crate version in {path:?}").into())
}

/// SemVer's breaking-release signal. For 0.y.z crates, a minor bump is the
/// breaking line Cargo users understand; for 1+ crates, require a major bump.
fn is_breaking_version_bump(old: CrateVersion, new: CrateVersion) -> bool {
    if old.major == 0 {
        new.major > 0 || (new.major == 0 && new.minor > old.minor)
    } else {
        new.major > old.major
    }
}

// --------------------------------------------------------------------------
// regen-tables: dump-free regeneration of the PHF tables from the committed
// lockfiles (the same path check-registry's sync gate verifies). Useful when
// only the EMISSION changes (e.g. the key format), not the lock content.
// --------------------------------------------------------------------------
fn regen_tables() -> Result<(), Box<dyn Error>> {
    let assignments_dir = default_assignments_dir()?;
    let generated_dir = default_generated_dir()?;
    let (noun, verb, adj) = load_locks(&assignments_dir)?;
    // A structurally invalid lock (hand-edit, bad merge) must never reach the
    // emitter: the pad-with-default column path would happily write "" inflections.
    let invalid: Vec<String> = noun
        .validate()
        .into_iter()
        .chain(verb.validate())
        .chain(adj.validate())
        .collect();
    if !invalid.is_empty() {
        return Err(format!(
            "refusing to regenerate tables — {} structural violation(s) in the lockfiles:\n  - {}",
            invalid.len(),
            invalid.join("\n  - ")
        )
        .into());
    }
    generate_tables(&noun, &verb, &adj, &generated_dir)?;
    println!(
        "Regenerated tables in {} from the lockfiles.",
        generated_dir.display()
    );
    Ok(())
}

// --------------------------------------------------------------------------
// report-coverage: how much of each lock rests on the durable sid: tier (which
// survives form drift in place) vs. the content sig: tier (which retires on
// drift and needs a reviewed override to rematch).
// --------------------------------------------------------------------------
fn report_coverage() -> Result<(), Box<dyn Error>> {
    let assignments_dir = default_assignments_dir()?;
    let (noun, verb, adj) = load_locks(&assignments_dir)?;
    for (name, lock) in [("noun", &noun), ("verb", &verb), ("adj", &adj)] {
        let c = lock.coverage();
        println!(
            "{name:>5}: active={} (sid={} sig={} legacy={})  tombstoned={}",
            c.active, c.by_sid, c.by_sig, c.legacy, c.tombstoned
        );
    }
    Ok(())
}

/// `git show <rev:path>`; `Ok(None)` ONLY when the path does not exist at that
/// revision (a genuinely new file). Any other git failure — bad object, missing
/// tree, corrupt/partial clone — is a hard error: callers treat `None` as
/// "skip the cross-version check", and the immutability gate must never fail
/// open because git itself failed.
fn git_show(root: &Path, target: &str) -> Result<Option<String>, Box<dyn Error>> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["show", target])
        .output()?;
    if out.status.success() {
        return Ok(Some(String::from_utf8_lossy(&out.stdout).into_owned()));
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("does not exist") || stderr.contains("exists on disk, but not in") {
        return Ok(None);
    }
    Err(format!("`git show {target}` failed: {}", stderr.trim()).into())
}

/// `git rev-parse --verify <rev>`; `None` when the rev cannot be resolved.
fn git_rev_parse(root: &Path, rev: &str) -> Result<Option<String>, Box<dyn Error>> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "--verify", rev])
        .output()?;
    if out.status.success() {
        let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok((!sha.is_empty()).then_some(sha))
    } else {
        Ok(None)
    }
}

/// Parse a lockfile from an in-memory CSV string (via a temp file, reusing Lock::load).
fn load_lock_from_str(csv: &str) -> Result<Lock, Box<dyn Error>> {
    // Process-unique dir so concurrent/stale runs can't read each other's temp file.
    let dir = env::temp_dir().join(format!("english_xtask_headlock_{}", process::id()));
    fs::create_dir_all(&dir)?;
    let path = dir.join("head.lock.csv");
    fs::write(&path, csv)?;
    let lock = Lock::load(&path);
    let _ = fs::remove_dir_all(&dir);
    lock
}

fn req(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, Box<dyn Error>> {
    iter.next()
        .ok_or_else(|| format!("expected a value after `{flag}`").into())
}

// --------------------------------------------------------------------------
// refresh-data: unchanged orchestration, now passing through the new flags.
// --------------------------------------------------------------------------
fn refresh_data(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let parsed = parse_refresh_data_args(args)?;

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let root = workspace_root()?;

    let mut command = Command::new(cargo);
    command.current_dir(&root);
    command
        .arg("run")
        .arg("-p")
        .arg("extractor")
        .arg("--release");
    if parsed.with_checks {
        command.arg("--features").arg("checks");
    }
    command.arg("--");
    command.arg("--dump").arg(&parsed.dump_path);

    if let Some(generated_dir) = parsed.generated_dir {
        command.arg("--generated-dir").arg(generated_dir);
    }
    if let Some(artifacts_dir) = parsed.artifacts_dir {
        command.arg("--artifacts-dir").arg(artifacts_dir);
    }
    if let Some(assignments_dir) = parsed.assignments_dir {
        command.arg("--assignments-dir").arg(assignments_dir);
    }
    if let Some(data_date) = parsed.data_date {
        command.arg("--data-date").arg(data_date);
    }
    if parsed.with_checks {
        command.arg("--run-checks");
    }
    if parsed.allow_partial_refresh {
        command.arg("--allow-partial-refresh");
    }

    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        process::exit(status.code().unwrap_or(1));
    }
}

fn parse_refresh_data_args(args: Vec<String>) -> Result<RefreshDataArgs, Box<dyn Error>> {
    let mut dump_path = None;
    let mut generated_dir = None;
    let mut artifacts_dir = None;
    let mut assignments_dir = None;
    let mut data_date = None;
    let mut with_checks = false;
    let mut allow_partial_refresh = false;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dump" => dump_path = Some(PathBuf::from(req(&mut iter, "--dump")?)),
            "--generated-dir" => {
                generated_dir = Some(PathBuf::from(req(&mut iter, "--generated-dir")?))
            }
            "--artifacts-dir" => {
                artifacts_dir = Some(PathBuf::from(req(&mut iter, "--artifacts-dir")?))
            }
            "--assignments-dir" => {
                assignments_dir = Some(PathBuf::from(req(&mut iter, "--assignments-dir")?))
            }
            "--data-date" => data_date = Some(req(&mut iter, "--data-date")?),
            "--with-checks" => with_checks = true,
            "--allow-partial-refresh" => allow_partial_refresh = true,
            "-h" | "--help" => {
                print_refresh_data_usage();
                process::exit(0);
            }
            _ if arg.starts_with("--") => {
                return Err(format!("unknown flag for `refresh-data`: {arg}").into());
            }
            _ => {
                if dump_path.is_some() {
                    return Err("expected a single dump path".into());
                }
                dump_path = Some(PathBuf::from(arg));
            }
        }
    }

    let dump_path = dump_path.ok_or(
        "missing dump path. Use `cargo xtask refresh-data --dump /path/to/rawwiki.jsonl`.",
    )?;

    Ok(RefreshDataArgs {
        dump_path,
        generated_dir,
        artifacts_dir,
        assignments_dir,
        data_date,
        with_checks,
        allow_partial_refresh,
    })
}

fn print_usage() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  refresh-data       Regenerate the Wiktionary-derived lookup tables + lockfiles");
    eprintln!("  seed-assignments   One-time: freeze today's tables into the assignment lockfiles");
    eprintln!("  check-registry     CI gate: fail if any published key changed meaning");
    eprintln!("  accuracy           Measure % of attested Wiktionary forms the library reproduces");
    eprintln!(
        "  compact            Data-major release: drop tombstones, close suffix gaps, bump DATA_MAJOR"
    );
    eprintln!("  regen-tables       Dump-free: regenerate the PHF tables from the lockfiles");
    eprintln!(
        "  migrate-anchors    One-time: re-anchor legacy qid/etym rows to form signatures (bumps DATA_MAJOR)"
    );
    eprintln!("  report-coverage    Show anchor-tier coverage of the lockfiles");
}

fn print_refresh_data_usage() {
    eprintln!(
        "Usage: cargo xtask refresh-data --dump /path/to/rawwiki.jsonl [--data-date YYYY-MM-DD] [--with-checks]"
    );
    eprintln!("       cargo xtask refresh-data /path/to/rawwiki.jsonl");
}
