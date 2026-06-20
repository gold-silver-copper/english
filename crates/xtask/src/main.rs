use extractor::bootstrap::{generate_tables, load_locks, seed};
use extractor::helpers::Pos;
use extractor::registry::{Lock, check_immutability};
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
                eprintln!("Usage: cargo xtask seed-assignments [--data-date YYYY-MM-DD] [--generated-dir DIR] [--assignments-dir DIR]");
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

    let mut violations = Vec::new();

    // During the pre-release window the sense keys are still being re-keyed freely,
    // so cross-version immutability is intentionally relaxed via ENGLISH_ALLOW_RELOCK
    // (set in CI). Internal consistency and lock<->table sync are ALWAYS enforced.
    // Unset this (the default) when cutting the first gated release to arm the
    // cross-version immutability guard.
    let allow_relock = env::var("ENGLISH_ALLOW_RELOCK")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);

    // Baseline = the commit this branch diverged from on the target branch, so we
    // gate the change THIS branch/PR makes to the lockfiles. Comparing against HEAD
    // is useless under CI (the checked-out working tree IS HEAD), so a key swap that
    // arrives inside a PR would slip through. We use the merge-base with the PR base
    // ref (GITHUB_BASE_REF on GitHub, else origin/main), which needs full history
    // (actions/checkout with fetch-depth: 0).
    let baseline = if allow_relock { None } else { resolve_baseline(&root)? };
    if allow_relock {
        println!(
            "check-registry: ENGLISH_ALLOW_RELOCK set — cross-version immutability SKIPPED \
             (pre-release relock window); internal consistency + table sync still enforced."
        );
    } else {
        match &baseline {
            Some(rev) => println!("check-registry: gating lock changes against baseline {rev}"),
            None => println!(
                "note: no baseline ref resolved (origin/main / GITHUB_BASE_REF); running internal + sync checks only."
            ),
        }
    }

    for (pos, working) in [(Pos::Noun, &noun), (Pos::Verb, &verb), (Pos::Adj, &adj)] {
        // 1. Internal consistency (unique anchors, no two identities share a suffix)
        //    plus strict structural validation (suffix>=1, anchor<->fields, no emit
        //    key collisions). Always enforced, never relaxed by the relock window.
        violations.extend(check_immutability(working, working));
        violations.extend(working.validate());

        // 2. Immutability vs. the baseline lock, if present there.
        if let Some(rev) = &baseline {
            let rel = format!("data/assignments/{}.lock.csv", pos.as_str());
            match git_show(&root, &format!("{rev}:{rel}"))? {
                Some(csv) => {
                    let base_lock = load_lock_from_str(&csv)?;
                    violations.extend(check_immutability(&base_lock, working));
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

    if violations.is_empty() {
        println!("check-registry: OK — no published key changed meaning; tables match the lock.");
        Ok(())
    } else {
        eprintln!("check-registry: FAILED — {} violation(s):", violations.len());
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
    let candidates: Vec<String> = match &base_ref {
        Some(b) => vec![format!("origin/{b}"), b.clone()],
        None => vec!["origin/main".to_string(), "main".to_string()],
    };

    for base in &candidates {
        if let Some(mb) = git_merge_base(root, base, "HEAD")? {
            return Ok(Some(mb));
        }
    }

    // No baseline resolved. Under CI this is fail-open and must NOT pass: a shallow
    // checkout, a missing base fetch, or a renamed default branch would otherwise
    // silently disable the headline cross-version immutability check while CI stays
    // green. Locally (no CI) it's fine to skip the cross-version comparison.
    let in_ci = base_ref.is_some() || env::var("CI").is_ok();
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
// report-coverage: how much of each lock rests on strong vs. weak anchors.
// --------------------------------------------------------------------------
fn report_coverage() -> Result<(), Box<dyn Error>> {
    let assignments_dir = default_assignments_dir()?;
    let (noun, verb, adj) = load_locks(&assignments_dir)?;
    for (name, lock) in [("noun", &noun), ("verb", &verb), ("adj", &adj)] {
        let c = lock.coverage();
        println!(
            "{name:>5}: active={} (qid={} sid={} etym={} sig={})  tombstoned={}",
            c.active, c.by_qid, c.by_sid, c.by_etym, c.by_sig, c.tombstoned
        );
    }
    Ok(())
}

/// `git show <rev:path>`; returns None if the object does not exist.
fn git_show(root: &Path, target: &str) -> Result<Option<String>, Box<dyn Error>> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["show", target])
        .output()?;
    if out.status.success() {
        Ok(Some(String::from_utf8_lossy(&out.stdout).into_owned()))
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
    iter.next().ok_or_else(|| format!("expected a value after `{flag}`").into())
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

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dump" => dump_path = Some(PathBuf::from(req(&mut iter, "--dump")?)),
            "--generated-dir" => generated_dir = Some(PathBuf::from(req(&mut iter, "--generated-dir")?)),
            "--artifacts-dir" => artifacts_dir = Some(PathBuf::from(req(&mut iter, "--artifacts-dir")?)),
            "--assignments-dir" => {
                assignments_dir = Some(PathBuf::from(req(&mut iter, "--assignments-dir")?))
            }
            "--data-date" => data_date = Some(req(&mut iter, "--data-date")?),
            "--with-checks" => with_checks = true,
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
    })
}

fn print_usage() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  refresh-data       Regenerate the Wiktionary-derived lookup tables + lockfiles");
    eprintln!("  seed-assignments   One-time: freeze today's tables into the assignment lockfiles");
    eprintln!("  check-registry     CI gate: fail if any published key changed meaning");
    eprintln!("  report-coverage    Show anchor-tier coverage of the lockfiles");
}

fn print_refresh_data_usage() {
    eprintln!("Usage: cargo xtask refresh-data --dump /path/to/rawwiki.jsonl [--data-date YYYY-MM-DD] [--with-checks]");
    eprintln!("       cargo xtask refresh-data /path/to/rawwiki.jsonl");
}
