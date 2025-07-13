# english

A high-performance English inflection library written in Rust. This crate provides functions to **conjugate verbs** and **decline nouns/adjectives** for English. It was built by processing large English Wiktionary datasets to capture nearly all irregular forms. The implementation heavily uses Rust’s **metaprogramming (macros)** to generate efficient, data-driven code at compile time. Internally, it stores inflection rules in pre-sorted arrays and uses binary search for fast lookup. Importantly, it has **no external dependencies**, making it extremely lightweight and easy to embed in other projects.

## Features

* **English only:** Supports only English grammar (verbs, nouns, adjectives).
* **Wiktionary-based data:** Covers regular and irregular forms using extracted Wiktionary data.
* **Fast lookups:** Uses sorted tables and binary search for efficient inflection.
* **Zero dependencies:** No runtime dependencies (zero-cost to include).
* **Metaprogramming:** Rust macros generate the inflection logic at compile time.
* **Game and NLP friendly:** Ideal for game dialogue systems or any text-processing task requiring correct English forms.

## Example Usage

```rust
use english::English;
use english::Tense;
use english::Number;

fn main() {
    // Conjugate a verb (handles irregulars)
    let past = English::verb("go", Tense::Past);
    println!("go (past) -> {}", past); // went

    // Decline a noun (handles irregulars)
    let plural = English::noun("child", Number::Plural);
    println!("child (plural) -> {}", plural); // children

    // Regular forms
    assert_eq!(English::verb("walk", Tense::Past), "walked");
    assert_eq!(English::noun("cat", Number::Plural), "cats");
}
```

In this code, calling `English::verb` or `English::noun` returns the correctly inflected form (even for irregular words). The library’s compile-time data and lookup tables ensure these calls are very fast.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
english = "0.0.3"
```

Then import and use it as shown above. Because the crate has **no runtime dependencies**, you can embed it in any project without adding extra libraries.

## License

This library is released under the MIT License.

**Sources:** Design and implementation are based on wiktionary data processing, Rust macros (metaprogramming), and a no-dependencies philosophy to maximize performance and portability.


## Inspirations
https://github.com/atteo/evo-inflector
