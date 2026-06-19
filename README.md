# english

[![Crates.io](https://img.shields.io/crates/v/english)](https://crates.io/crates/english)
[![Docs.rs](https://docs.rs/english/badge.svg)](https://docs.rs/english)
![License](https://img.shields.io/crates/l/english)
[![Discord](https://img.shields.io/discord/123456789012345678.svg?logo=discord&logoColor=white&color=5865F2)](https://discord.gg/tDBPkdgApN)


**english** is a blazing fast and light weight English inflection library written in Rust. Total bundled data size is less than 1 MB. It provides extremely accurate verb conjugation and noun/adjective declension based on highly processed Wiktionary data, making it ideal for real-time procedural text generation.

## ⚡ Speed and Accuracy

Evaluation of the English inflector (`extractor/main.rs/check_*`) and performance benchmarking (`examples/speedmark.rs`) shows:

| Part of Speech | Correct / Total | Accuracy  | Throughput (calls/sec) | Time per Call |
|----------------|----------------|-----------|-----------------------|---------------|
| **Nouns**      | 243495 / 244001 | 99.79%   | 7,499,749             | 133 ns        |
| **Verbs**      | 161215 / 165457 | 97.44%   | 12,423,891            | 80 ns         |
| **Adjectives** | 121512 / 121719 | 99.83%   | 15,607,807            | 64 ns         |

*Note: Benchmarking was done under a worst-case scenario; typical real-world usage is 50~ nanoseconds faster.*

## 📦 Installation

```
cargo add english
```

Then in your code:

```rust
use english::*;
fn main() {
    // --- Mixed Sentence Example ---
    let subject_number = Number::Plural;
    let subject = format!(
        "{} {}",
        English::verb(
            "run",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle
        ),
        English::noun("child", &subject_number)
    ); // running children
    let verb = English::verb(
        "steal",
        &Person::Third,
        &subject_number,
        &Tense::Past,
        &Form::Finite,
    ); //stole
    let object = count_with_number("potato", 7); //7 potatoes

    let sentence = format!("The {} {} {}.", subject, verb, object);
    assert_eq!(sentence, "The running children stole 7 potatoes.");

    // --- Nouns ---
    assert_eq!(
        format!("{} of jeans", count_with_number("pair", 3)),
        "3 pairs of jeans"
    );
    // Regular plurals
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(English::noun("die2", &Number::Plural), "dice");
    // Use count function for better ergonomics if needed
    assert_eq!(count("man", 2), "men");
    // Use count_with_number function to preserve the number
    assert_eq!(count_with_number("nickel", 3), "3 nickels");
    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // --- Verbs ---
    assert_eq!(
        English::verb(
            "pick",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "picked"
    );
    assert_eq!(
        English::verb(
            "walk",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Participle
        ),
        "walking"
    );
    assert_eq!(
        English::verb(
            "go",
            &Person::First,
            &Number::Singular,
            &Tense::Past,
            &Form::Participle
        ),
        "gone"
    );
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(
        English::verb(
            "lie",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lay"
    );
    assert_eq!(
        English::verb(
            "lie2",
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lied"
    );
    // "to be" has the most verb forms in english and requires using verb()
    assert_eq!(
        English::verb(
            "be",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite
        ),
        "am"
    );

    // --- Adjectives ---
    // Add a number 2-9 to the end of the word to try different forms. (Bad has the most forms at 3)
    assert_eq!(English::adj("bad", &Degree::Comparative), "more bad");
    assert_eq!(English::adj("bad", &Degree::Superlative), "most bad");
    assert_eq!(English::adj("bad2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad2", &Degree::Superlative), "baddest");
    assert_eq!(English::adj("bad3", &Degree::Comparative), "worse");
    assert_eq!(English::adj("bad3", &Degree::Superlative), "worst");
    assert_eq!(English::adj("bad3", &Degree::Positive), "bad");

    // --- Pronouns ---
    assert_eq!(
        English::pronoun(
            &Person::First,
            &Number::Singular,
            &Gender::Neuter,
            &Case::PersonalPossesive
        ),
        "my"
    );
    assert_eq!(
        English::pronoun(
            &Person::First,
            &Number::Singular,
            &Gender::Neuter,
            &Case::Possessive
        ),
        "mine"
    );

    // --- Possessives ---
    assert_eq!(English::add_possessive("dog"), "dog's");
    assert_eq!(English::add_possessive("dogs"), "dogs'");
}
```

---

For a more involved but still minimal example of building a small domain layer on top of `english`, see `crates/english/examples/semantic_triples.rs`:

```bash
cargo run -p english --example semantic_triples
```

It shows custom noun/verb/adj/adv types, semantic triples, perspective-sensitive rendering, modifiers, complements, adjuncts, and agreement-driven pronoun and tense shifts.

## 🔧 Crate Overview

### `english`

> The public API for verb conjugation and noun/adjective declension.

* Combines optimized data generated from `extractor` with inflection logic from `english-core`
* Pure Rust, only one dependency: phf
* PHF-backed irregular lookups with regular-rule fallback
* Code generation ensures no runtime penalty

### `english-core`

> The core engine for English inflection — pure algorithmic logic.

* Implements the core rules for conjugation/declension
* Used to classify forms as regular or irregular for the extractor
* Has no data dependency — logic-only
* Can be used stand alone for an even smaller footprint (at the cost of some accuracy)

### `extractor`

> A tool to process and refine Wiktionary data.

* Parses large English Wiktionary dumps
* Extracts all verb, noun, and adjective forms
* Uses `english-core` to filter out regular forms, preserving only irregulars
* Resolves homograph sense numbers against the checked-in assignment lockfiles so keys stay stable across releases
* Generates the static PHF tables used in `english`

---

## 📦 Obtaining Wiktionary Data & Running the Extractor

This project relies on raw data extracted from Wiktionary. Current version built with data from 2026-06-19.

- [Wiktextract (GitHub)](https://github.com/tatuylonen/wiktextract)
- [Kaikki.org raw data](https://kaikki.org/dictionary/rawdata.html)

### Steps

1. Download the **raw Wiktextract JSONL dump** (~20 GB) from [Kaikki.org](https://kaikki.org/dictionary/rawdata.html).
2. Place the file somewhere accessible (e.g. `../rawwiki.jsonl`).
3. From the repository root, run: `cargo xtask refresh-data --dump ../rawwiki.jsonl --data-date YYYY-MM-DD` (use the dump's date)
4. The generated Rust tables are written to `/crates/english/generated`, the assignment lockfiles to `/data/assignments`, and intermediate CSV/JSONL artifacts to `/data/intermediate`.
5. Review `git diff data/assignments/` like a `Cargo.lock` diff, then run `cargo xtask check-registry` before committing.

To also run the extractor evaluation reports against the current library data, add `--with-checks`.

## 🔒 Deterministic sense numbering

Homographs that inflect differently share a lemma and are disambiguated by a numeric suffix
(`lie` → *lay*, `lie2` → *lied*; `die2` → *dice*). Historically that suffix was **positional** —
assigned by alphabetically sorting whatever forms survived filtering in a given dump — so any
upstream change could renumber a lemma and silently swap the meaning of a published key.

The suffix is now pinned in checked-in **assignment lockfiles** (`data/assignments/{noun,verb,adj}.lock.csv`),
the data equivalent of `Cargo.lock`:

* Each emitted key is anchored to a stable identity (Wikidata QID → senseid → `etymology_number` →,
  as a last resort, the form signature) and a **frozen** suffix.
* On every refresh, senses are matched to existing lock rows by anchor and **reuse** their pinned
  suffix (forms may update in place); genuinely new senses get **append-only** higher suffixes;
  vanished senses are **tombstoned** and their suffix is retired forever.
* `cargo xtask check-registry` fails the build if any previously-committed key changed meaning.
* `cargo xtask report-coverage` shows how much of each lockfile rests on strong vs. weak anchors.

Discover which numbered senses exist for a lemma instead of hard-coding suffixes
(enable the optional `senses` feature, which bundles the variant tables — kept off
by default so the shipped data stays under 1 MB):

```toml
english = { version = "0.2", features = ["senses"] }
```

```rust
use english::English;
assert_eq!(English::verb_senses("lie"), &["lie", "lie2"]);
assert_eq!(English::noun_senses("die"), &["die2"]);
assert!(English::noun_senses("cat").is_empty());
```

The lockfiles were originally seeded from the shipped tables with `cargo xtask seed-assignments`
(a one-time bootstrap); you should not need to run it again.

## Benchmarks
Performance benchmarks were run on my M2 Macbook.

Writing benchmarks and tests for such a project is rather difficult and requires opinionated decisions. Many words may have alternative inflections, and the data in wiktionary is not perfect. Many words might be both countable and uncountable, the tagging of words may be inconsistent. This library includes a few uncountable words in its dataset, but not all. Uncountable words require special handling anyway. Take all benchmarks with a pound of salt, write your own tests for your own usecases. Any suggestions to improve the benchmarking are highly appreciated.

## Disclaimer
Wiktionary data is often unstable and subject to weird changes. This means that the provided inflections may change unexpectedly. The generated lookup tables in `crates/english/generated/*_phf.rs` are the source of truth for a given revision, and the assignment lockfiles in `data/assignments/*.lock.csv` keep each sense-numbered key (`lie2`, `die2`, …) pointing at the same inflection across revisions — see [Deterministic sense numbering](#-deterministic-sense-numbering).

## Inspirations and Thanks
- Ole in the bevy discord suggested I use ```phf``` instead of sorted arrays, this resulted in up to 40% speedups
- https://github.com/atteo/evo-inflector
- https://github.com/plurals/pluralize


## 📄 License

- Code: Dual licensed under MIT and Apache © 2024 [gold-silver-copper](https://github.com/gold-silver-copper)
  - [MIT](https://opensource.org/licenses/MIT)
  - [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)

- Data: Wiktionary content is dual-licensed under
  - [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
  - [GNU FDL](https://www.gnu.org/licenses/fdl-1.3.html)
