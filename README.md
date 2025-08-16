# english

[![Crates.io](https://img.shields.io/crates/v/english)](https://crates.io/crates/english)
[![Docs.rs](https://docs.rs/english/badge.svg)](https://docs.rs/english)
![License](https://img.shields.io/crates/l/english)

**english** is a blazing fast English morphology library written in Rust with zero external dependencies. It provides accurate verb conjugation and noun/adjective declension based on processed Wiktionary data, making it ideal for procedural text generation.

## Speed and Accuracy
In-sample evaluation reveals the following accuracy of the english inflector (extractor/main/check_*):

Nouns: 235719 / 236150 plurals correctly guessed (99.82%)

Verbs: 154711 / 156474 distinct verb forms correctly guessed (98.87%)

Adjectives: 118136 / 118221 comparative and superlative forms correctly guessed (99.92%)

Preliminary performance benchmarking reveals the following speeds of the english inflector (examples/speedmark.rs):

[verb] Throughput: 8949672 calls/sec | Time per call: 111.74 ns
[noun] Throughput: 6245139 calls/sec | Time per call: 160.12 ns
[adjective] Throughput: 9448375 calls/sec | Time per call: 105.84 ns



## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
english = "0.0.4"
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

    // Invariant nouns
    assert_eq!(English::noun("sheep", &Number::Plural), "sheep");

    // --- Adjectives ---
    // Regular adjectives
    assert_eq!(English::adj("fast", &Degree::Comparative), "faster");

    // Irregular adjectives
    // Add a number 2-9 to the end of the word to try different forms.
    assert_eq!(English::adj("bad", &Degree::Comparative), "more bad");
    assert_eq!(English::adj("bad", &Degree::Superlative), "most bad");
    assert_eq!(English::adj("bad2", &Degree::Comparative), "badder");
    assert_eq!(English::adj("bad2", &Degree::Superlative), "baddest");
    assert_eq!(English::adj("bad3", &Degree::Comparative), "worse");
    assert_eq!(English::adj("bad3", &Degree::Superlative), "worst");

    // --- Verbs ---
    // Regular verbs
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
    let subject = English::noun("child", &Number::Plural);
    let verb = English::verb(
        "play",
        &Person::Third,
        &Number::Plural,
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
* Fast binary search and compact data structures
* Tiny binary footprint, perfect for embedded usage

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

## ✨ Features

* ✅ High-accuracy inflection from real-world Wiktionary data
* 🚀 Extremely fast: uses pre-sorted static arrays with binary search
* ⚙️ Metaprogrammed: static sorted arrays generated at compile time
* 🧩 Zero external dependencies — fully self-contained
* 📦 Tiny and embeddable
* 🧠 Ideal for procedural text generation

## ⚡ Performance

* Code generation ensures no runtime penalty.
* Irregular forms stored in static slices.
* Binary search over pre-sorted arrays: `O(log n)` lookup.

This makes `english` suitable for high-performance tasks like:

* Procedural text generation for games or other interactive media
* NLP or AI pipelines

## Benchmarks
In-sample evaluation reveals the following accuracy of the english inflector.

Nouns: 235719 / 236150 plurals correctly guessed (99.82%)

Verbs: 154711 / 156474 distinct verb forms correctly guessed (98.87%)

Adjectives: 118136 / 118221 comparative and superlative forms correctly guessed (99.92%)

Writing benchmarks for such a project is rather difficult and required opinionated decisions. Many words may have alternative inflections, and the data in wiktionary is not perfect. Many words might be both countable and uncountable, the tagging of words may be inconsistent. This library includes a few uncountable words in its dataset, but not all. Uncountable words require special handling anyway. Any suggestions to improve the benchmarking are highly appreciated.

## Obtaining Wiktionary Data and running the extractor
https://github.com/tatuylonen/wiktextract

https://kaikki.org/dictionary/rawdata.html

Download the raw wiktextract data from the kaikki website. In the extractor file point the functions to use the raw data.

## Inspirations
https://github.com/atteo/evo-inflector

## 📄 License

- Code: Dual licensed under MIT and Apache © 2024 [gold-silver-copper](https://github.com/gold-silver-copper)
  - [MIT](https://opensource.org/licenses/MIT)
  - [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)

- Data: Wiktionary content is dual-licensed under
  - [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)
  - [GNU FDL](https://www.gnu.org/licenses/fdl-1.3.html)
