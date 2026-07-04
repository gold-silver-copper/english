//! Workspace task runner (`cargo xtask <command>`).
//!
//! - `refresh-data` — full dump-driven regeneration of the PHF tables (delegates
//!   to the extractor binary in release mode);
//! - `check-registry` — the dump-free CI gate: the committed tables are
//!   well-formed, have unique keys and correct column arity, and preserve
//!   rule/table layering (a bare key never merely duplicates the rule engine);
//! - `accuracy` — dump-driven measurement of rules + tables together.
//!
//! Sense-numbered `_<n>` keys are DETERMINISTIC but NOT immutable: `refresh-data`
//! regenerates every table from the dump alone (see `extractor::assign`), so an
//! upstream data change can renumber a lemma's keys. There is no lockfile,
//! override file, human-review flow, or cross-version immutability gate — the
//! committed generated tables are the whole artifact.

use extractor::bootstrap::audit_tables;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::{self, Command};

struct RefreshDataArgs {
    dump_path: PathBuf,
    artifacts_dir: Option<PathBuf>,
}

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

fn default_generated_dir() -> Result<PathBuf, Box<dyn Error>> {
    Ok(workspace_root()?.join("crates/english/generated"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("refresh-data") => refresh_data(args.collect()),
        Some("check-registry") => check_registry(),
        Some("accuracy") => accuracy(args.collect()),
        Some("-h") | Some("--help") | None => {
            print_usage();
            Ok(())
        }
        Some(command) => Err(format!("unknown xtask command: {command}").into()),
    }
}

// --------------------------------------------------------------------------
// check-registry: dump-free structural + layering gate over the committed
// tables. Fails if a key is malformed/duplicated, a row has the wrong arity or
// an empty column, or a bare key merely duplicates the rule engine (a rule
// changed without a regenerate). No lockfile, no cross-version immutability, no
// overrides. It CANNOT verify a row's irregular VALUES are correct/current —
// those are attested data, not derivable without the dump; `cargo xtask
// accuracy` (with the dump) is the authoritative value check.
// --------------------------------------------------------------------------
fn check_registry() -> Result<(), Box<dyn Error>> {
    let generated_dir = default_generated_dir()?;
    let violations = audit_tables(&generated_dir)?;
    if violations.is_empty() {
        println!(
            "check-registry: OK — tables are well-formed, keys are unique, and rule/table layering \
             holds. (Structural + layering only; run `cargo xtask accuracy` to verify the forms \
             against the dump.)"
        );
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

// --------------------------------------------------------------------------
// accuracy: measure "% of plain attested Wiktionary forms the library
// reproduces" per POS, against the CURRENT committed tables + rule engine.
// Defaults to the cached filtered dataset so it doesn't need the raw dump.
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

fn req(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, Box<dyn Error>> {
    iter.next()
        .ok_or_else(|| format!("expected a value after `{flag}`").into())
}

// --------------------------------------------------------------------------
// refresh-data: run the extractor over a dump, regenerating all PHF tables.
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
        .arg("--release")
        .arg("--");
    command.arg("--dump").arg(&parsed.dump_path);
    if let Some(artifacts_dir) = parsed.artifacts_dir {
        command.arg("--artifacts-dir").arg(artifacts_dir);
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
    let mut artifacts_dir = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dump" => dump_path = Some(PathBuf::from(req(&mut iter, "--dump")?)),
            "--artifacts-dir" => {
                artifacts_dir = Some(PathBuf::from(req(&mut iter, "--artifacts-dir")?))
            }
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
        artifacts_dir,
    })
}

fn print_usage() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  refresh-data     Regenerate the Wiktionary-derived PHF tables from a dump");
    eprintln!("  check-registry   Dump-free gate: tables well-formed, keys unique, layering holds");
    eprintln!("  accuracy         Measure % of attested Wiktionary forms the library reproduces");
}

fn print_refresh_data_usage() {
    eprintln!(
        "Usage: cargo xtask refresh-data --dump /path/to/rawwiki.jsonl [--artifacts-dir DIR]"
    );
    eprintln!("       cargo xtask refresh-data /path/to/rawwiki.jsonl");
}
