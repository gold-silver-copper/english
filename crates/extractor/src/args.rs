//! CLI contract for the extractor binary (normally invoked via
//! `cargo xtask refresh-data`, which forwards these flags verbatim).
//!
//! The generated tables always go to the committed layout
//! (`crates/english/generated`) — that is the only location `check-registry`, the
//! runtime, and the accuracy harness read. The filtered-dump cache defaults to
//! `data/intermediate` and can be redirected with `--artifacts-dir`.

use std::env;
use std::error::Error;
use std::path::PathBuf;

const DEFAULT_GENERATED_DIR: &str = "crates/english/generated";
const DEFAULT_ARTIFACTS_DIR: &str = "data/intermediate";

#[derive(Debug, Clone)]
pub struct Config {
    pub dump_path: PathBuf,
    /// Where the PHF tables are written — always the committed layout.
    pub generated_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    /// Measure accuracy against the currently-compiled committed tables, then
    /// stop without regenerating (`cargo xtask accuracy`). Note this reflects the
    /// tables compiled into THIS binary, not any tables a refresh would produce.
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
    let generated_dir = repo_root.join(DEFAULT_GENERATED_DIR);
    let mut artifacts_dir = repo_root.join(DEFAULT_ARTIFACTS_DIR);
    let mut checks_only = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dump" => {
                let value = args.next().ok_or("expected a path after `--dump`")?;
                dump_path = Some(PathBuf::from(value));
            }
            "--artifacts-dir" => {
                let value = args
                    .next()
                    .ok_or("expected a path after `--artifacts-dir`")?;
                artifacts_dir = PathBuf::from(value);
            }
            "--checks-only" => {
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
        checks_only,
    })
}

pub fn print_usage() {
    eprintln!(
        "Usage: cargo run -p extractor --release -- --dump /path/to/rawwiki.jsonl [--artifacts-dir DIR] [--checks-only]"
    );
    eprintln!("       cargo run -p extractor --release -- /path/to/rawwiki.jsonl");
}
