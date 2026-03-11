#[cfg(feature = "checks")]
use crate::helpers::{Entry, base_setup, contains_bad_tag, entry_is_proper, word_is_proper};
use std::error::Error;
#[cfg(feature = "checks")]
use std::io::BufRead;
use std::path::Path;

pub fn run_checks(filtered_json_path: &Path, artifacts_dir: &Path) -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "checks")]
    {
        check_noun_plurals(
            filtered_json_path,
            artifacts_dir.join("noun_plural_check.csv"),
        )?;
        check_verb_conjugations(filtered_json_path, artifacts_dir.join("verbs_check.csv"))?;
        check_adjective_forms(filtered_json_path, artifacts_dir.join("adj_check.csv"))?;
        Ok(())
    }

    #[cfg(not(feature = "checks"))]
    {
        let _ = filtered_json_path;
        let _ = artifacts_dir;
        Err("extractor was built without the `checks` feature. Re-run with `cargo xtask refresh-data --dump /path/to/rawwiki.jsonl --with-checks`.".into())
    }
}

#[cfg(feature = "checks")]
fn numbered_variants(word: &str) -> Vec<String> {
    let mut variants = vec![word.to_string()];
    for i in 2..=9 {
        variants.push(format!("{word}{i}"));
    }
    variants
}

#[cfg(feature = "checks")]
pub fn check_noun_plurals(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    use english::*;

    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["wiki_single", "wiktionary_plural"])?;

    let mut total_counter = 0;
    let mut match_counter = 0;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry_is_proper(&entry, "noun") {
            continue;
        }
        let lowercased_entry = entry.word.to_lowercase();

        let mut wiktionary_plurals = Vec::new();
        if let Some(forms) = entry.forms {
            for form in forms {
                if form.tags.contains(&"plural".into()) {
                    wiktionary_plurals.push(form.form.to_lowercase());
                }
            }
        }
        if wiktionary_plurals.is_empty() {
            continue;
        }

        let variants = numbered_variants(&lowercased_entry);
        for wiki_plural in &wiktionary_plurals {
            let wiki_plural = wiki_plural.to_lowercase();
            total_counter += 1;
            let mut matched = false;

            for variant in &variants {
                let generated_plural = English::noun(variant, &Number::Plural);
                matched = generated_plural == wiki_plural;
                if matched {
                    match_counter += 1;
                    break;
                }
            }

            if !matched {
                writer.write_record(&[lowercased_entry.clone(), wiki_plural.clone()])?;
            }
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path.display());
    println!("total match amount: {} / {}", match_counter, total_counter);
    Ok(())
}

#[cfg(feature = "checks")]
pub fn check_verb_conjugations(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    use english::*;

    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path, output_path);

    writer.write_record(&["wiktionary_form", "person", "number", "tense", "form"])?;

    let mut total_counter = 0;
    let mut match_counter = 0;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry_is_proper(&entry, "verb") {
            continue;
        }

        let lowercased_entry = entry.word.to_lowercase();
        let mut wiktionary_forms = Vec::new();

        if let Some(forms) = entry.forms {
            for form in forms {
                let tags = form
                    .tags
                    .iter()
                    .map(|tag| tag.to_lowercase())
                    .collect::<Vec<_>>();
                let form_str = form.form.to_lowercase();

                if form_str == "dubious"
                    || contains_bad_tag(form.tags.clone())
                    || !word_is_proper(&form.form)
                {
                    continue;
                }

                let person = if tags.contains(&"first-person".into()) {
                    continue;
                } else if tags.contains(&"second-person".into()) {
                    continue;
                } else {
                    Person::Third
                };

                let number = if tags.contains(&"plural".into()) {
                    continue;
                } else {
                    Number::Singular
                };

                let tense = if tags.contains(&"present".into()) {
                    Tense::Present
                } else if tags.contains(&"past".into()) {
                    Tense::Past
                } else {
                    Tense::Present
                };

                let form_type = if tags.contains(&"participle".into()) {
                    Form::Participle
                } else if tags.contains(&"infinitive".into()) {
                    continue;
                } else {
                    Form::Finite
                };

                wiktionary_forms.push((form_str, person, number, tense, form_type));
            }
        }

        if wiktionary_forms.is_empty() {
            continue;
        }

        let variants = numbered_variants(&lowercased_entry);
        for (wiki_form, person, number, tense, form_type) in wiktionary_forms {
            total_counter += 1;
            let mut matched = false;

            for variant in &variants {
                let generated_form = English::verb(variant, &person, &number, &tense, &form_type);
                matched = generated_form == wiki_form;
                if matched {
                    match_counter += 1;
                    break;
                }
            }

            if !matched {
                writer.write_record(&[
                    wiki_form.clone(),
                    format!("{:?}", person),
                    format!("{:?}", number),
                    format!("{:?}", tense),
                    format!("{:?}", form_type),
                ])?;
            }
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path.display());
    println!("total match amount: {} / {}", match_counter, total_counter);
    Ok(())
}

#[cfg(feature = "checks")]
pub fn check_adjective_forms(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    use english::*;

    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let (reader, mut writer) = base_setup(input_path, output_path);
    writer.write_record(&["wiktionary_form", "degree"])?;

    let mut total_counter = 0;
    let mut match_counter = 0;

    for line in reader.lines() {
        let line = line?;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry_is_proper(&entry, "adj") {
            continue;
        }

        let lowercased_entry = entry.word.to_lowercase();
        let mut wiki_comparative = None;
        let mut wiki_superlative = None;

        if let Some(forms) = entry.forms {
            for form in forms {
                let form_str = form.form.to_lowercase();
                let tags_lower: Vec<String> =
                    form.tags.iter().map(|tag| tag.to_lowercase()).collect();

                if tags_lower.contains(&"comparative".into()) {
                    wiki_comparative = Some(form_str);
                } else if tags_lower.contains(&"superlative".into()) {
                    wiki_superlative = Some(form_str);
                }
            }
        }

        if wiki_comparative.is_none() && wiki_superlative.is_none() {
            continue;
        }

        let variants = numbered_variants(&lowercased_entry);

        if let Some(wiki_comp) = &wiki_comparative {
            let wiki_comp = wiki_comp.to_lowercase();
            total_counter += 1;
            let mut matched = false;

            for variant in &variants {
                let generated_comp = English::adj(variant, &Degree::Comparative);
                if generated_comp == wiki_comp {
                    match_counter += 1;
                    matched = true;
                    break;
                }
            }

            if !matched {
                writer.write_record(&[wiki_comp.clone(), "Comparative".into()])?;
            }
        }

        if let Some(wiki_sup) = &wiki_superlative {
            let wiki_sup = wiki_sup.to_lowercase();
            total_counter += 1;
            let mut matched = false;

            for variant in &variants {
                let generated_sup = English::adj(variant, &Degree::Superlative);
                if generated_sup == wiki_sup {
                    match_counter += 1;
                    matched = true;
                    break;
                }
            }

            if !matched {
                writer.write_record(&[wiki_sup.clone(), "Superlative".into()])?;
            }
        }
    }

    writer.flush()?;
    println!("Done! Output written to {}", output_path.display());
    println!("total match amount: {} / {}", match_counter, total_counter);
    Ok(())
}
