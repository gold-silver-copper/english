# english

[![Crates.io](https://img.shields.io/crates/v/english)](https://crates.io/crates/english)
[![Docs.rs](https://docs.rs/english/badge.svg)](https://docs.rs/english)
![License](https://img.shields.io/crates/l/english)
[![Discord](https://img.shields.io/discord/123456789012345678.svg?logo=discord&logoColor=white&color=5865F2)](https://discord.gg/tDBPkdgApN)


**english** is a blazing fast English morphology library written in Rust with zero external dependencies and a total code+data size under 1 MB. It provides extremely accurate verb conjugation and noun/adjective declension based on highly processed Wiktionary data, making it ideal for real-time procedural text generation.

## ⚡ Speed and Accuracy

### Accuracy
In-sample evaluation of the English inflector (`extractor/main.rs/check_*`) shows:

| Part of Speech | Correct / Total | Accuracy |
|----------------|-----------------|-----------|
| **Nouns**      | 238106 / 238549 | **99.81%** |
| **Verbs**      | 158056 / 161643 | **97.78%** |
| **Adjectives** | 119200 / 119356 | **99.86%** |

---

### Performance
Benchmarking under a worst-case scenario (`examples/speedmark.rs`) yields the following speeds:

| Part of Speech | Throughput (calls/sec) | Time per Call |
|----------------|-------------------------|---------------|
| **Verbs**      | 5,572,956               | 180 ns     |
| **Nouns**      | 3,668,747               | 272 ns     |
| **Adjectives** | 7,167,281               | 139 ns     |
Note: Real world use cases are generally 50-100 nanoseconds faster

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
english = "0.0.7"
```

Then in your code:

```rust
use english::*;
fn main() {
    // --- Nouns ---
    // Regular plurals
    assert_eq!(English::noun("cat", &Number::Plural), "cats");

    // Irregular plurals
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(English::noun("child", &Number::Plural), "children");
    assert_eq!(English::noun("die", &Number::Plural), "dies");
    assert_eq!(English::noun("die2", &Number::Plural), "dice");

    // Use count function for better ergonomics if needed
    assert_eq!(English::count("man", 2), "men");
    // Use count_with_number function to preserve the number
    assert_eq!(English::count_with_number("nickel", 3), "3 nickels");

    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // Complex nouns, note that From<&str> is impl'd for Noun
    // Note that noun(), count(), etc can work on both strings and Noun struct
    let jeans = Noun::from("pair").with_complement("of jeans");
    assert_eq!(English::count_with_number(jeans, 3), "3 pairs of jeans");

    // --- Adjectives ---
    // Add a number 2-9 to the end of the word to try different forms. (Bad has the most forms at 3)
    assert_eq!(English::adj("bad", &Degree::Comparative), "more bad");
    assert_eq!(English::adj("bad", &Degree::Superlative), "most bad");
    assert_eq!(English::adj("bad2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad2", &Degree::Superlative), "baddest");
    assert_eq!(English::adj("bad3", &Degree::Comparative), "worse");
    assert_eq!(English::adj("bad3", &Degree::Superlative), "worst");

    // Complex verbs, note that From<&str> is impl'd for Verb
    let pick_up = Verb::from("pick").with_particle("up");

    // Past tense, third person singular
    assert_eq!(
        English::verb(
            &pick_up,
            &Person::Third,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "picked up"
    );

    // --- Verbs ---
    // Regular verbs, note that verb() can be used on both strings and Verb struct
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

    // Irregular verbs
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
    assert_eq!(
        English::verb(
            "go",
            &Person::Third,
            &Number::Plural,
            &Tense::Past,
            &Form::Participle
        ),
        "gone"
    );
    assert_eq!(
        English::verb(
            "lie",
            &Person::First,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lay"
    );
    assert_eq!(
        English::verb(
            "lie2",
            &Person::First,
            &Number::Singular,
            &Tense::Past,
            &Form::Finite
        ),
        "lied"
    );

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

    // --- Mixed Sentence Example ---
    let subject_number = Number::Plural;
    let subject = English::noun("child", &subject_number);
    let verb = English::verb(
        "play",
        &Person::Third,
        &subject_number,
        &Tense::Past,
        &Form::Finite,
    );
    let object = English::noun("die2", &Number::Plural);

    let sentence = format!("The {} {} with {}.", subject, verb, object);
    assert_eq!(sentence, "The children played with dice.");
}
```

---

## 🔧 Crate Overview

### `english`

> The public API for verb conjugation and noun/adjective declension.

* Combines optimized data generated from `extractor` with inflection logic from `english-core`
* Pure Rust, no external dependencies
* Fast Binary search over pre-sorted arrays: `O(log n)` lookup.
* Code generation ensures no runtime penalty.

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
* Generates sorted static arrays for use in `english`

---

## 📦 Obtaining Wiktionary Data & Running the Extractor

This project relies on raw data extracted from Wiktionary. Current version built with data from 8/17/2025.

- [Wiktextract (GitHub)](https://github.com/tatuylonen/wiktextract)
- [Kaikki.org raw data](https://kaikki.org/dictionary/rawdata.html)

### Steps

1. Download the **raw Wiktextract JSONL dump** (~20 GB) from [Kaikki.org](https://kaikki.org/dictionary/rawdata.html).
2. Place the file somewhere accessible (e.g. `../rawwiki.jsonl`).
3. From the `extractor` folder, run: `cargo run --release ../rawwiki.jsonl`
4. Move the generated files adj_array.rs, noun_array.rs, verb_array.rs into the /src of english

## Benchmarks
Performance benchmarks were run on my M2 Macbook.

Writing benchmarks and tests for such a project is rather difficult and requires opinionated decisions. Many words may have alternative inflections, and the data in wiktionary is not perfect. Many words might be both countable and uncountable, the tagging of words may be inconsistent. This library includes a few uncountable words in its dataset, but not all. Uncountable words require special handling anyway. Take all benchmarks with a pound of salt, write your own tests for your own usecases. Any suggestions to improve the benchmarking are highly appreciated.

## Disclaimer
Wiktionary data is often unstable and subject to weird changes. This means that the provided inflections may change unexpectedly. You can look at the diffs of *_array.rs files for a source of truth.

## Inspirations
- https://github.com/atteo/evo-inflector
- https://github.com/plurals/pluralize

## 📄 License

- Code: Dual licensed under MIT and Apache © 2024 [gold-silver-copper](https://github.com/gold-silver-copper)
  - [MIT](https://opensource.org/licenses/MIT)
  - [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)

- Data: Wiktionary content is dual-licensed under
  - [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
  - [GNU FDL](https://www.gnu.org/licenses/fdl-1.3.html)
