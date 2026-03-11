use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::{self, Command};

struct RefreshDataArgs {
    dump_path: PathBuf,
    generated_dir: Option<PathBuf>,
    artifacts_dir: Option<PathBuf>,
    with_checks: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("refresh-data") => refresh_data(args.collect()),
        Some("-h") | Some("--help") | None => {
            print_usage();
            Ok(())
        }
        Some(command) => Err(format!("unknown xtask command: {command}").into()),
    }
}

fn refresh_data(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let parsed = parse_refresh_data_args(args)?;

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;

    let mut command = Command::new(cargo);
    command.current_dir(&workspace_root);
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
    let mut with_checks = false;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dump" => {
                let value = iter.next().ok_or("expected a path after `--dump`")?;
                dump_path = Some(PathBuf::from(value));
            }
            "--generated-dir" => {
                let value = iter
                    .next()
                    .ok_or("expected a path after `--generated-dir`")?;
                generated_dir = Some(PathBuf::from(value));
            }
            "--artifacts-dir" => {
                let value = iter
                    .next()
                    .ok_or("expected a path after `--artifacts-dir`")?;
                artifacts_dir = Some(PathBuf::from(value));
            }
            "--with-checks" => {
                with_checks = true;
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
        generated_dir,
        artifacts_dir,
        with_checks,
    })
}

fn print_usage() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  refresh-data    Regenerate the Wiktionary-derived lookup tables");
}

fn print_refresh_data_usage() {
    eprintln!("Usage: cargo xtask refresh-data --dump /path/to/rawwiki.jsonl [--with-checks]");
    eprintln!("       cargo xtask refresh-data /path/to/rawwiki.jsonl");
}
