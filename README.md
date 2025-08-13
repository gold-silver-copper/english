# english

[![Crates.io](https://img.shields.io/crates/v/english)](https://crates.io/crates/english)
[![Docs.rs](https://docs.rs/english/badge.svg)](https://docs.rs/english)
![License](https://img.shields.io/crates/l/english)

**english** is a blazing fast English morphology library written in Rust. It provides accurate verb conjugation and noun/adjective declension based on processed Wiktionary data, making it ideal for language processing and game development.

This repository contains multiple tightly integrated crates working together to deliver an efficient, dependency-free inflection engine. It was built by processing large English Wiktionary datasets to capture nearly all irregular forms. The implementation heavily uses Rust’s **metaprogramming (macros)** to generate efficient, data-driven code at compile time. Internally, it stores inflection rules in pre-sorted arrays and uses binary search for fast lookup. Importantly, it has **no external dependencies**, making it extremely lightweight and easy to embed in other projects.

---

## 🔧 Crate Overview

### `english`

> The public API for verb conjugation and noun/adjective declension.

* Combines optimized data generated from `extractor` with inflection logic from `english-core`
* Pure Rust, no dependencies
* Fast binary search and compact data structures
* Tiny binary footprint, perfect for embedded usage

### `english-core`

> The core engine for English inflection — pure algorithmic logic.

* Implements the core rules for conjugation/declension
* Used to classify forms as regular or irregular
* Has no data dependency — logic-only
* Can be used stand alone for an even smaller footprint (at the cost of some accuracy)

### `extractor`

> A tool to process and refine Wiktionary data.

* Parses large English Wiktionary dumps
* Extracts all verb, noun, and adjective forms
* Uses `english-core` to filter out regular forms, preserving only irregulars
* Generates compact tables for use in `english`

---

## ✨ Features

* ✅ High-accuracy inflection from real-world Wiktionary data
* 🚀 Extremely fast: uses pre-sorted static arrays with binary search
* ⚙️ Metaprogrammed: static sorted arrays generated at compile time
* 🧩 Zero external dependencies — fully self-contained
* 📦 Tiny, embeddable, and ready for production
* 🧠 Ideal for NLP pipelines and game dialogue engines

---

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
    // Conjugate a verb (handles irregulars)
    let past = English::verb(
        "eat",
        &Person::Third,
        &Number::Singular,
        &Tense::Past,
        &Form::Finite,
    );
    println!("eat (past) -> {}", past); // ate

    // Decline a noun (handles irregulars)
    let plural = English::noun("child", &Number::Plural);
    println!("child (plural) -> {}", plural); // children

    // Regular forms
    assert_eq!(English::noun("cat", &Number::Plural), "cats");
}

```

---

## ⚡ Performance

* Compile-time macro expansion ensures no runtime penalty.
* Irregular forms stored in compact static slices.
* Binary search over pre-sorted data: `O(log n)` lookup.
* Minimal memory usage and no heap allocation.

This makes `english` suitable for high-performance or embedded environments like:

* Dialogue trees in games
* Procedural text generators
* Edge devices or WASM


---

## 📄 License

MIT License © 2024 [gold-silver-copper](https://github.com/gold-silver-copper)


## Inspirations
https://github.com/atteo/evo-inflector


## Benchmarks
https://github.com/monolithpl/verb.forms.dictionary
