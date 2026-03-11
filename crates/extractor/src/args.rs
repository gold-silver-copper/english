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
    })
}

pub fn print_usage() {
    eprintln!(
        "Usage: cargo run -p extractor --release -- --dump /path/to/rawwiki.jsonl [--generated-dir generated] [--artifacts-dir data/intermediate] [--run-checks]"
    );
    eprintln!("       cargo run -p extractor --release -- /path/to/rawwiki.jsonl");
}
