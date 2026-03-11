use crate::args::Config;
use crate::checks::run_checks;
use crate::extract::{
    extract_irregular_adjectives, extract_irregular_nouns, extract_verb_conjugations,
    filter_english_entries,
};
use crate::file_generation::{generate_adjectives_phf, generate_nouns_phf, generate_verbs_phf};
use std::error::Error;
use std::fs;

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&config.generated_dir)?;
    fs::create_dir_all(&config.artifacts_dir)?;

    let filtered_json_path = config.artifacts_dir.join("english_filtered.jsonl");
    if config.dump_path != filtered_json_path {
        filter_english_entries(&config.dump_path, &filtered_json_path)?;
    } else {
        println!(
            "Reusing filtered dataset at {}",
            filtered_json_path.display()
        );
    }

    if config.run_checks {
        run_checks(&filtered_json_path, &config.artifacts_dir)?;
    }

    let verbs_csv = config.artifacts_dir.join("verb_conjugations.csv");
    let nouns_csv = config.artifacts_dir.join("nouns_with_plurals.csv");
    let adjectives_csv = config.artifacts_dir.join("adjectives.csv");

    extract_verb_conjugations(&filtered_json_path, &verbs_csv)?;
    extract_irregular_nouns(&filtered_json_path, &nouns_csv)?;
    extract_irregular_adjectives(&filtered_json_path, &adjectives_csv)?;

    generate_nouns_phf(&nouns_csv, config.generated_dir.join("noun_phf.rs"))?;
    generate_adjectives_phf(&adjectives_csv, config.generated_dir.join("adj_phf.rs"))?;
    generate_verbs_phf(&verbs_csv, config.generated_dir.join("verb_phf.rs"))?;

    Ok(())
}
