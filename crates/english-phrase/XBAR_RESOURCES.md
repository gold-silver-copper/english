# `english-phrase` DP and X-bar resources

## Purpose

This document collects the most useful resources for developing `english-phrase` with a **DP-first X-bar backbone**.

The key commitment is:

- X-bar gives the structural template
- **DP is the maximal nominal projection**
- NP survives only as the internal nominal core

So the goal is not “how do we model NP?” but:

> how do we model DP, VP, PP, AdjP, AdvP, and Clause in a way that generates good English?

## What X-bar gives us

X-bar theory is useful here as a structural template for code.

It gives a clean way to think about:

- head
- complement
- specifier
- adjunct / modifier
- projection

That maps naturally to phrase objects and clause objects.

For `english-phrase`, the immediate payoff is:

- **DP** gets a determiner layer above a nominal core
- **VP** gets a head, complements, and adjuncts
- **PP** gets a preposition head and a typed complement
- **AdjP** and **AdvP** get heads, modifiers, and complements
- **Clause** sits above phrase structure

## Best source stack

If you want the best overall stack, use:

1. **Carnie and introductory syntax texts** for X-bar itself
2. **Abney** for the DP layer
3. **Penn Treebank** for concrete phrase inventories and bracketing
4. **Cambridge Grammar Today** for open English examples
5. **CGEL / Quirk** for high-authority descriptive decisions

That combination is much better than relying on any single source.

---

## Core X-bar sources

### 1. Andrew Carnie

Best use:

- learning the structural logic of X-bar
- understanding head / complement / specifier / adjunct relations
- grounding the phrase API in standard syntax rather than intuition

Sources:

- [Constituent Structure, chapter “X-bar Theory”](https://academic.oup.com/book/48037/chapter-abstract/421291005)
- [Carnie textbook/workbook page](https://www.andrewcarnie.org/copy-of-textbook4e)
- [The Syntax Workbook](https://www.wiley-vch.de/en/areas-interest/humanities-social-sciences/linguistics-12lg/theoretical-linguistics-12lg1/syntax-12lg11/the-syntax-workbook-978-1-119-56929-9)

Why this matters:

- It is the cleanest starting point for phrase-structure design.
- It is especially helpful for deciding what the internal shape of DP, VP, PP, and Clause should be.

### 2. UBC X-bar theory notes

Best use:

- quick review
- implementation-oriented summary of phrase projection and constituent roles

Source:

- [UBC X-bar Theory wiki](https://wiki.ubc.ca/Course%3ALING300/X-bar_Theory)

### 3. SDSU notes on complements, specifiers, and adjuncts

Best use:

- deciding what belongs in phrase structure vs clause structure
- clarifying complement / specifier / adjunct distinctions

Source:

- [Complements, Specifiers, and Adjuncts](https://gawron.sdsu.edu/syntax/new_midterm/adj_comp_spec.htm)

### 4. Phrase inventory slides for nominal and modifier phrases

Best use:

- quick structural comparison across nominal, adjectival, adverbial, and prepositional phrases

Source:

- [General structure of NP, AdjP, AdvP, and PP](https://dept.english.wisc.edu/rfyoung/324/NPAdjPAdvPandPP.pdf)

Important note:

- The slide deck still uses **NP** because that is standard in many pedagogical materials.
- For this project, read that material through a **DP-first** lens: NP is the nominal core, not the maximal nominal phrase.

---

## DP resources

### What to model

For the nominal layer, a DP-oriented X-bar model should include:

- determiner
- possessive
- quantifier
- numeral
- nominal premodifiers
- nominal head
- nominal complements
- postmodifiers
- relative clauses
- apposition

### Best sources

1. [Abney’s DP work](https://mitwpl.mit.edu/catalog/abne01)
2. [CORE mirror / metadata for Abney 1987](https://core.ac.uk/display/24457458)
3. [Glottolog metadata entry](https://glottolog.org/resource/reference/id/29190)
4. [Penn Treebank II guidelines](https://repository.upenn.edu/handle/20.500.14332/6998)
5. [Cambridge noun phrase pages](https://dictionary.cambridge.org/us/grammar/british-grammar/phrases)
6. CGEL / Quirk

### Why these matter

- Abney is the key source for taking D seriously.
- PTB helps with actual constituency patterns, even if it still labels them NP.
- Cambridge is excellent for examples and test material.
- CGEL helps decide what belongs in the determiner layer, nominal layer, and postmodifier layer.

### Future API direction

```rust
let dp = DeterminerPhrase::new("child")
    .determiner(Determiner::the())
    .quantifier(Quantifier::many())
    .modifier(AdjectivePhrase::new("small").comparative())
    .postmodifier(
        PrepositionalPhrase::new(
            "from",
            DeterminerPhrase::new("building").the(),
        )
    )
    .relative_clause(
        RelativeClause::who(
            Clause::new(
                DeterminerPhrase::gap(),
                VerbPhrase::new("wait").past().simple().affirmative(),
            )
        )
    )
    .plural();
```

---

## VP resources

### What to model

For verb phrases, a useful X-bar-oriented inventory includes:

- lexical head verb
- auxiliary chain
- particles
- complements
- obliques
- adjuncts
- predicative complements
- clause complements
- voice and finiteness

### Best sources

1. Carnie
2. Aarts
3. CGEL
4. Cambridge verb phrase pages
5. Penn Treebank

### Future API direction

```rust
let vp = VerbPhrase::new("give")
    .past()
    .simple()
    .affirmative()
    .indirect_object(DeterminerPhrase::new("child").the())
    .direct_object(DeterminerPhrase::new("apple").an())
    .oblique(
        PrepositionalPhrase::new(
            "after",
            DeterminerPhrase::new("class").the(),
        )
    );
```

---

## PP resources

### What to model

For prepositional phrases:

- preposition head
- typed complement
- PP recursion
- PP attachment to DP, VP, AdjP, and Clause
- PP complements vs PP adjuncts

### Best sources

1. SDSU complement/specifier/adjunct notes
2. Cambridge prepositional phrase pages
3. CGEL
4. Penn Treebank

### Future API direction

```rust
let pp = PrepositionalPhrase::new(
    "on",
    DeterminerPhrase::new("wall")
        .the()
        .postmodifier(
            PrepositionalPhrase::new(
                "inside",
                DeterminerPhrase::new("hall").the(),
            )
        ),
);
```

---

## AdjP resources

### What to model

For adjective phrases:

- degree
- intensifiers
- PP complements
- infinitival complements
- comparative complements
- attributive vs predicative position

### Best sources

1. Cambridge adjective phrase pages
2. CGEL
3. Quirk
4. X-bar teaching materials

### Future API direction

```rust
let ap = AdjectivePhrase::new("bad")
    .comparative()
    .intensifier("far")
    .complement("than yesterday");
```

---

## AdvP resources

### What to model

For adverb phrases:

- head adverb
- degree modifiers
- clause-level use
- VP-level use
- adjective and adverb modification

### Best sources

1. Cambridge adverb phrase page
2. CGEL
3. Quirk
4. phrase-structure teaching materials

### Future API direction

```rust
let advp = AdverbPhrase::new("very")
    .modifier("surprisingly");
```

---

## Clause resources

### What to model

For clauses:

- subject DP
- VP predicate
- direct object
- indirect object
- obliques
- adjuncts
- clause type
- finite vs non-finite
- relative clauses
- complement clauses

### Best sources

1. CGEL
2. Aarts
3. Carnie
4. Cambridge clause pages
5. Penn Treebank

### Future API direction

```rust
let clause = Clause::new(
    DeterminerPhrase::new("child").the().plural(),
    VerbPhrase::new("steal")
        .past()
        .simple()
        .affirmative()
        .direct_object(DeterminerPhrase::new("potato").count(7)),
)
.oblique(
    PrepositionalPhrase::new(
        "from",
        DeterminerPhrase::new("market").the(),
    )
);

assert_eq!(clause.render(), "the children stole 7 potatoes from the market");
```

### Finite vs non-finite clause note

A finite clause carries finiteness:

- `the children left`
- `the child has eaten`

A non-finite clause does not:

- `to leave`
- `leaving early`
- `having eaten`

This matters because non-finite clauses naturally appear inside other phrases:

- DP: `the decision to leave`
- AdjP: `ready to leave`
- PP: `before leaving`

---

## Best practical reading order

If you want the most useful sequence for this project:

1. Carnie on X-bar
2. Abney on DP
3. SDSU notes on complements/specifiers/adjuncts
4. Penn Treebank II guidelines
5. Cambridge phrase and clause pages
6. CGEL when design questions become hard

## Best three sources if you want to move fast

If you only want three:

1. [Carnie on constituent structure / X-bar](https://academic.oup.com/book/48037/chapter-abstract/421291005)
2. [Abney’s DP work](https://mitwpl.mit.edu/catalog/abne01)
3. [Penn Treebank II guidelines](https://repository.upenn.edu/handle/20.500.14332/6998)

That combination gives you:

- phrase-structure theory
- a real determiner-headed nominal model
- a practical inventory of English phrase variants
