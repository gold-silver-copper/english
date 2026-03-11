/*{  let word_key = match entry.etymology_number {
    Some(1) => infinitive.clone(),
    Some(x) => format!("{infinitive}{x}"),
    None => infinitive.clone(),
};
}
 */

/*if plural_found {
    let mut form_count = 1;

    match forms_set.remove(&predicted_plural) {
        true => {}
        false => {}
    }

    for formik in forms_set {
        let gotten = [infinitive.clone(), formik.clone()];
    }

    let gotten = [infinitive.clone(), forms_map.get("plural").unwrap().clone()];
    let keyd_struct = [word_key.clone(), forms_map.get("plural").unwrap().clone()];

    if predicted_struct == gotten {
        duplicate_pairs_set.insert(predicted_struct.clone());
    }

    if !duplicate_key_set.contains(&word_key) && !duplicate_pairs_set.contains(&gotten) {
        duplicate_key_set.insert(word_key.clone());
        duplicate_pairs_set.insert(gotten.clone());
        writer.write_record(&keyd_struct)?;
    }
} */

/*

pub fn generate_nouns_file(inputik: &str, outputik: &str) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let mut pairs: Vec<(String, String)> = reader
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(',');
            Some((
                parts.next()?.trim().to_string(),
                parts.next()?.trim().to_string(),
            ))
        })
        .collect();

    // Sort by the word (key)
    pairs.sort_by_key(|(word, _)| word.clone());

    // Write to a Rust file
    let mut output = File::create(outputik)?;

    writeln!(output, "static PLURAL_MAP: &[(&str, &str)] = &[")?;
    for (word, plural) in &pairs {
        writeln!(output, "    (\"{}\", \"{}\"),", word, plural)?;
    }
    writeln!(output, "];\n")?;

    writeln!(
        output,
        "pub fn get_plural(word: &str) -> Option<&'static str> {{"
    )?;
    writeln!(
        output,
        "    PLURAL_MAP.binary_search_by_key(&word, |&(k, _)| k).ok().map(|i| PLURAL_MAP[i].1)"
    )?;
    writeln!(output, "}}")?;
    Ok(())
}

pub fn generate_verbs_file(inputik: &str, outputik: &str) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let mut entries: Vec<(String, (String, String, String, String))> = reader
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(',');
            Some((
                parts.next()?.trim().to_string(), // infinitive
                (
                    parts.next()?.trim().to_string(), // 3rd person singular
                    parts.next()?.trim().to_string(), // past
                    parts.next()?.trim().to_string(), // present participle
                    parts.next()?.trim().to_string(), // past participle
                ),
            ))
        })
        .collect();

    // Sort by infinitive
    entries.sort_by_key(|(inf, _)| inf.clone());

    let mut output = File::create(outputik)?;

    writeln!(
        output,
        "/// (3rd person singular, past, present participle, past participle)"
    )?;
    writeln!(
        output,
        "static VERB_MAP: &[(&str, (&str, &str, &str, &str))] = &["
    )?;
    for (inf, (third, past, pres_part, past_part)) in &entries {
        writeln!(
            output,
            "    (\"{}\", (\"{}\", \"{}\", \"{}\", \"{}\")),",
            inf, third, past, pres_part, past_part
        )?;
    }
    writeln!(output, "];\n")?;

    writeln!(
        output,
        "pub fn get_verb_forms(infinitive: &str) -> Option<(&'static str, &'static str, &'static str, &'static str)> {{"
    )?;
    writeln!(
        output,
        "    VERB_MAP.binary_search_by_key(&infinitive, |&(k, _)| k)"
    )?;
    writeln!(output, "        .ok()")?;
    writeln!(output, "        .map(|i| VERB_MAP[i].1)")?;
    writeln!(output, "}}")?;

    Ok(())
}

pub fn generate_adjectives_file(inputik: &str, outputik: &str) -> std::io::Result<()> {
    let input = File::open(inputik)?;
    let reader = BufReader::new(input);

    let mut entries: Vec<(String, (String, String))> = reader
        .lines()
        .skip(1) // Skip header row
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(',');
            Some((
                parts.next()?.trim().to_string(), // positive
                (
                    parts.next()?.trim().to_string(), // comparative
                    parts.next()?.trim().to_string(), // superlative
                ),
            ))
        })
        .collect();

    // Sort by positive form
    entries.sort_by_key(|(pos, _)| pos.clone());

    let mut output = File::create(outputik)?;

    writeln!(output, "/// (comparative, superlative)")?;
    writeln!(output, "static ADJECTIVE_MAP: &[(&str, (&str, &str))] = &[")?;
    for (positive, (comparative, superlative)) in &entries {
        writeln!(
            output,
            "    (\"{}\", (\"{}\", \"{}\")),",
            positive, comparative, superlative
        )?;
    }
    writeln!(output, "];\n")?;

    writeln!(
        output,
        "pub fn get_adjective_forms(positive: &str) -> Option<(&'static str, &'static str)> {{"
    )?;
    writeln!(
        output,
        "    ADJECTIVE_MAP.binary_search_by_key(&positive, |&(k, _)| k)"
    )?;
    writeln!(output, "        .ok()")?;
    writeln!(output, "        .map(|i| ADJECTIVE_MAP[i].1)")?;
    writeln!(output, "}}")?;

    Ok(())
}
 */
