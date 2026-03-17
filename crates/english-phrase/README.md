# english-phrase

`english-phrase` builds compositional English phrase structure on top of
[`english`](https://docs.rs/english).

It separates lexical categories (`NP`, `DP`, `VP`, `TP`, `CP`) from
realization, so the same structure can be surfaced as plain text or as a
sentence with capitalization and punctuation.

## Highlights

- `cp(...)` now makes force explicit with `.content()` and `.relative()`
- common public surface types have semantic aliases such as `ContentClause`,
  `RelativeClause`, `SingularDeterminerPhrase`, and `PluralDeterminerPhrase`
- agreement and gap information stay encoded in the builder API, while the
  public surface remains readable

## Example

```rust
use english_phrase::*;

let subject: SingularDeterminerPhrase =
    dp(np("editor").modifier(adjp("patient"))).the();

let clause = tp(vp("admire").complement(dp(Pronoun::She)))
    .present()
    .subject(subject);

assert_eq!(
    clause.realize_with(RealizationOptions::sentence()),
    "The patient editor admires her."
);

let content: ContentClause = cp(tp(vp("admire").complement(dp(Pronoun::She)))
    .past()
    .subject(dp(Pronoun::He)))
.content()
.that();

assert_eq!(content.realize(), "that he admired her");

let relative: RelativeClause<ObjectGap> =
    cp(tp(vp("admire").object_gap()).past().subject(dp(name("Alice"))))
        .relative()
        .that();

assert_eq!(np("editor").relative(relative).realize(), "editor that Alice admired");
```
