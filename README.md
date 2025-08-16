# english

[![Crates.io](https://img.shields.io/crates/v/english)](https://crates.io/crates/english)
[![Docs.rs](https://docs.rs/english/badge.svg)](https://docs.rs/english)
![License](https://img.shields.io/crates/l/english)

**english** is a blazing fast English morphology library written in Rust with zero external dependencies. It provides accurate verb conjugation and noun/adjective declension based on processed Wiktionary data, making it ideal for real-time procedural text generation.

## ⚡ Speed and Accuracy

### Accuracy
In-sample evaluation of the English inflector (`extractor/main.rs/check_*`) shows:

| Part of Speech | Correct / Total | Accuracy |
|----------------|-----------------|-----------|
| **Nouns**      | 235,719 / 236,150 | **99.82%** |
| **Verbs**      | 154,711 / 156,474 | **98.87%** |
| **Adjectives** | 118,136 / 118,221 | **99.92%** |

---

### Performance
Preliminary benchmarking (`examples/speedmark.rs`) yields the following speeds:

| Part of Speech | Throughput (calls/sec) | Time per Call |
|----------------|-------------------------|---------------|
| **Verbs**      | 8,949,672               | 111.74 ns     |
| **Nouns**      | 6,245,139               | 160.12 ns     |
| **Adjectives** | 9,448,375               | 105.84 ns     |


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

## Benchmarks
Performance benchmarks were run on my M2 Max Macbook.

Writing benchmarks and tests for such a project is rather difficult and required opinionated decisions. Many words may have alternative inflections, and the data in wiktionary is not perfect. Many words might be both countable and uncountable, the tagging of words may be inconsistent. This library includes a few uncountable words in its dataset, but not all. Uncountable words require special handling anyway. Any suggestions to improve the benchmarking are highly appreciated.

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
