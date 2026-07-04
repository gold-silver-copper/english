//! CLI contract for the extractor binary (normally invoked via
//! `cargo xtask refresh-data`, which forwards these flags verbatim).
//!
//! Paths default to the workspace-relative layout (committed tables in
//! `crates/english/generated`, cache in `data/intermediate`) and are overridable
//! individually — the accuracy audit relies on redirecting them to a scratch
//! directory.

use std::env;
use std::error::Error;
use std::path::PathBuf;

const DEFAULT_GENERATED_DIR: &str = "crates/english/generated";
const DEFAULT_ARTIFACTS_DIR: &str = "data/intermediate";

#[derive(Debug, Clone)]
pub struct Config {
    pub dump_path: PathBuf,
    pub generated_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub run_checks: bool,
    /// Measure accuracy only: filter (or reuse) the dump and run the checks, but
    /// never touch the generated tables.
    pub checks_only: bool,
}

pub fn parse_args() -> Result<Config, Box<dyn Error>> {
    parse_args_from(env::args().skip(1))
}

pub fn parse_args_from<I>(args: I) -> Result<Config, Box<dyn Error>>
where
    I: IntoIterator<Item = String>,
{
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let mut dump_path = None;
    let mut generated_dir = repo_root.join(DEFAULT_GENERATED_DIR);
    let mut artifacts_dir = repo_root.join(DEFAULT_ARTIFACTS_DIR);
    let mut run_checks = false;
    let mut checks_only = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dump" => {
                let value = args.next().ok_or("expected a path after `--dump`")?;
                dump_path = Some(PathBuf::from(value));
            }
            "--generated-dir" => {
                let value = args
                    .next()
                    .ok_or("expected a path after `--generated-dir`")?;
                generated_dir = PathBuf::from(value);
            }
            "--artifacts-dir" => {
                let value = args
                    .next()
                    .ok_or("expected a path after `--artifacts-dir`")?;
                artifacts_dir = PathBuf::from(value);
            }
            "--run-checks" => {
                run_checks = true;
            }
            "--checks-only" => {
                run_checks = true;
                checks_only = true;
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            _ if arg.starts_with("--") => {
                return Err(format!("unknown flag: {arg}").into());
            }
            _ => {
                if dump_path.is_some() {
                    return Err("expected a single dump path".into());
                }
                dump_path = Some(PathBuf::from(arg));
            }
        }
    }

    let dump_path =
        dump_path.ok_or("missing dump path. Use `cargo run -p extractor --release -- --dump /path/to/rawwiki.jsonl`.")?;

    Ok(Config {
        dump_path,
        generated_dir,
        artifacts_dir,
        run_checks,
        checks_only,
    })
}

pub fn print_usage() {
    eprintln!(
        "Usage: cargo run -p extractor --release -- --dump /path/to/rawwiki.jsonl [--generated-dir DIR] [--artifacts-dir DIR] [--run-checks]"
    );
    eprintln!("       cargo run -p extractor --release -- /path/to/rawwiki.jsonl");
}
