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
| **Nouns**      | 238106 / 238549 | 99.81%   | 5,228,300             | 191 ns        |
| **Verbs**      | 158056 / 161643 | 97.78%   | 8,473,248             | 118 ns        |
| **Adjectives** | 119200 / 119356 | 99.86%   | 11,999,052             | 83 ns        |

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
        Verb::new("run").present_participle(),
        English::noun("child", &subject_number)
    ); // running children
    let verb = English::verb(
        "steal",
        &Person::Third,
        &subject_number,
        &Tense::Past,
        &Form::Finite,
    ); //stole
    let object = Noun::new("potato").count_with_number(7); //7 potatoes

    let sentence = format!("The {} {} {}.", subject, verb, object);
    assert_eq!(sentence, "The running children stole 7 potatoes.");

    // For higher-level phrase builders, use the `english-phrase` crate.

    // --- Nouns ---
    assert_eq!(
        format!("{} of jeans", Noun::new("pair").count_with_number(3)),
        "3 pairs of jeans"
    );
    // Regular plurals
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(Noun::new("die2").plural(), "dice");
    // Use count function for better ergonomics if needed
    assert_eq!(Noun::new("man").count(2), "men");
    // Use count_with_number function to preserve the number
    assert_eq!(Noun::new("nickel").count_with_number(3), "3 nickels");
    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // --- Verbs ---
    // Verb helper methods operate on the base lemma only.
    assert_eq!(Verb::new("pick").past(), "picked");
    assert_eq!(Verb::new("walk").present_participle(), "walking");
    assert_eq!(Verb::new("go").past_participle(), "gone");
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(Verb::new("lie").past(), "lay");
    assert_eq!(Verb::new("lie2").past(), "lied");
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
    assert_eq!(Adj::new("bad2").comparative(), "badder");
    assert_eq!(Adj::new("bad2").superlative(), "baddest");
    assert_eq!(Adj::new("bad3").comparative(), "worse");
    assert_eq!(Adj::new("bad3").superlative(), "worst");
    assert_eq!(Adj::new("bad3").positive(), "bad");

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
* Generates the static PHF tables used in `english`

---

## 📦 Obtaining Wiktionary Data & Running the Extractor

This project relies on raw data extracted from Wiktionary. Current version built with data from 8/17/2025.

- [Wiktextract (GitHub)](https://github.com/tatuylonen/wiktextract)
- [Kaikki.org raw data](https://kaikki.org/dictionary/rawdata.html)

### Steps

1. Download the **raw Wiktextract JSONL dump** (~20 GB) from [Kaikki.org](https://kaikki.org/dictionary/rawdata.html).
2. Place the file somewhere accessible (e.g. `../rawwiki.jsonl`).
3. From the repository root, run: `cargo xtask refresh-data --dump ../rawwiki.jsonl`
4. The generated Rust tables are written to `/crates/english/generated`, and intermediate CSV/JSONL artifacts are written to `/data/intermediate`

To also run the extractor evaluation reports against the current library data, add `--with-checks`.

## Benchmarks
Performance benchmarks were run on my M2 Macbook.

Writing benchmarks and tests for such a project is rather difficult and requires opinionated decisions. Many words may have alternative inflections, and the data in wiktionary is not perfect. Many words might be both countable and uncountable, the tagging of words may be inconsistent. This library includes a few uncountable words in its dataset, but not all. Uncountable words require special handling anyway. Take all benchmarks with a pound of salt, write your own tests for your own usecases. Any suggestions to improve the benchmarking are highly appreciated.

## Disclaimer
Wiktionary data is often unstable and subject to weird changes. This means that the provided inflections may change unexpectedly. The generated lookup tables in `crates/english/generated/*_phf.rs` are the source of truth for a given revision.

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
