use std::error::Error;

pub mod args;
pub mod bootstrap;
pub mod checks;
pub mod extract;
pub mod file_generation;
pub mod helpers;
pub mod pipeline;
pub mod registry;

pub use args::Config;

pub fn run_from_env() -> Result<(), Box<dyn Error>> {
    let config = args::parse_args()?;
    pipeline::run(&config)
}
