# X-bar Theory Phrase Structure Rewrite Rules under the DP Hypothesis

Compiled for `english-phrase`  
Prepared on March 14, 2026

## Scope and method

This document consolidates the phrase structure rewrite rules most commonly given for `CP`, `TP`, `vP`, `VP`, `DP`, `NP`, `PP`, `AP`, and `AdvP` under a **DP-Hypothesis, X-bar-theoretic** understanding of English phrase structure.

This is not a claim that every textbook or article uses every rule. It is a comparative reference built from recurring rule families across the sources consulted. Where sources diverge, the disagreement is stated explicitly.

The baseline schema assumed throughout is the strict X-bar template:

- `XP -> (Specifier) X'`
- `X' -> X (Complement)`
- `X' -> X' Adjunct` or `X' -> Adjunct X'`

Under the DP Hypothesis, `DP` is the maximal nominal projection and `NP` is the nominal core beneath it.

## Source abbreviations used in the table

| Code | Source |
|---|---|
| `CAR` | Carnie, *Syntax: A Generative Introduction*, plus related workbook/chapter material |
| `ABN` | Abney, *The English Noun Phrase in Its Sentential Aspect* (DP Hypothesis) |
| `UBC` | UBC X-bar theory notes |
| `SDSU` | SDSU notes on complements, specifiers, and adjuncts |
| `WISC` | Wisconsin phrase-structure slides for NP/AdjP/AdvP/PP |
| `MIT` | MIT syntax lecture notes/resources on CP structure and pronouns in DP |
| `RAD` | Radford, especially split-projection and `vP`-shell discussions |
| `HAE` | Haegeman, GB-era clause and phrase-structure framing |
| `CGEL` | Huddleston & Pullum, *The Cambridge Grammar of the English Language* |
| `PTB` | Penn Treebank article and bracketing guidelines |
| `CAM` | Cambridge Grammar Today pages used for English examples and complement patterns |
| `DEB` | Debnath / e-PG Pathshala-INFLIBNET X-bar teaching notes |

## Consolidated reference table of rewrite rules

| Category | Rewrite rule(s) | How it instantiates the X-bar schema | Variation / controversy | Framework / status | Cross-check sources |
|---|---|---|---|---|---|
| General X-bar schema | `XP -> (YP) X'`<br>`X' -> X (ZP)` | Specifier projects at XP level; complement merges closest to the head at X' level. | Some teaching sources linearize the specifier to the left by default for English; later Bare Phrase Structure recasts this without rewrite rules. | Core strict X-bar; not controversial inside textbook X-bar syntax. | `CAR`, `UBC`, `SDSU`, `DEB`, `WISC` |
| Adjunction schema | `X' -> X' Adjunct`<br>`X' -> Adjunct X'` | Adjuncts attach to an intermediate projection, not directly to the lexical head and not in the same structural position as complements. | Right-adjunction is common in English NP/VP notation; left-adjunction is common for degree/intensifier material in AP/AdvP. Some minimalist work abandons rewrite rules and treats adjunction via pair-Merge. | Strict X-bar core; directionality is category- and framework-sensitive. | `CAR`, `SDSU`, `DEB`, `WISC`, `CGEL` |
| CP (baseline) | `CP -> (Spec) C'`<br>`C' -> C TP` | CP is the maximal clausal projection; complementizer heads the clause and selects TP. | Older GB sources often write `IP` instead of `TP`: `CP -> (Spec) C'; C' -> C IP`. | Standard GB / textbook X-bar clause architecture. | `CAR`, `MIT`, `HAE`, `RAD`, `PTB` |
| CP (split-CP variation) | `CP -> ForceP ... FinP`<br>`Force' -> Force TopP/FocP/FinP`<br>`Fin' -> Fin TP` | The single CP layer is split into multiple functional projections above TP. | Common in cartographic and minimalist-adjacent work, but not required by conservative X-bar presentations. | Controversial / framework-dependent split projection. | `RAD`, `CAR`, `MIT` |
| TP (baseline) | `TP -> DP T'`<br>`T' -> T VP` | Subject DP occupies Spec,TP; T heads the clause and selects VP. | Many modern sources replace lower `VP` with `vP` in the complement of T. | Conservative X-bar / GB-style clause rule. | `CAR`, `DEB`, `MIT`, `HAE` |
| TP (with split verbal domain) | `TP -> DP T'`<br>`T' -> T vP` | Subject DP remains in Spec,TP; T selects a light-verb projection rather than VP directly. | Becomes standard once `vP` shells are assumed; absent from more conservative textbook versions. | GB/Minimalist-adjacent split-projection variant. | `RAD`, `CAR`, `MIT` |
| TP (further split variation) | `TP/AgrSP -> DP T'/AgrS'`<br>`T'/AgrS' -> T/AgrS NegP/AspP/vP` | The clause is decomposed into additional projections for agreement, negation, aspect, and related functional material. | Very framework-dependent; common in some Principles-and-Parameters and early minimalist work, not universal in contemporary pedagogical X-bar syntax. | Controversial / framework-dependent split functional spine. | `RAD`, `HAE`, `CAR` |
| vP | `vP -> DP v'`<br>`v' -> v VP` | External argument is introduced in Spec,vP; the lexical VP is the complement of the light verb. | Not used in conservative pre-shell X-bar syntax. Once adopted, it strongly affects TP and VP rules. | Framework-dependent but very common in contemporary generative syntax. | `RAD`, `CAR`, `MIT` |
| VP (baseline) | `VP -> V'`<br>`V' -> V (DP/PP/AP/CP)` | Lexical V heads the phrase; complements sit closest to the verb. | The category of the complement varies with the verb's lexical selection: DP object, PP complement, AP predicative complement, CP clausal complement, and so on. | Core strict X-bar verbal rule. | `CAR`, `CGEL`, `CAM`, `PTB`, `DEB` |
| VP (adjunction) | `V' -> V' PP`<br>`V' -> V' AdvP` | Adjunct PPs and adverbials attach to V' rather than occupying the complement slot. | Attachment height for adverbs and some PPs is often debated. | Core X-bar with well-known attachment ambiguities. | `CAR`, `SDSU`, `CGEL`, `PTB` |
| VP (shell / ditransitive variation) | `vP -> DP v'`<br>`v' -> v VP`<br>`VP -> V DP` | Ditransitive and causative structures are represented through a layered verbal domain rather than a single flat VP. | Highly dependent on whether a `vP`-shell analysis is assumed. | Framework-dependent split verbal architecture. | `RAD`, `CAR` |
| DP (canonical) | `DP -> (DP) D'`<br>`D' -> D NP` | D is the head of the maximal nominal projection; NP is its complement; possessor DP can occupy Spec,DP. | Some sources write `DP -> D'` when no overt specifier is present; the parenthesized initial DP is the possessor slot. | Canonical DP-Hypothesis X-bar rule. | `ABN`, `CAR`, `MIT`, `CGEL` |
| Possessor placement | `DP -> DP D'`<br>`D' -> D NP` | The possessor sits in Spec,DP; D heads the phrase and NP remains the nominal complement. | Some implementations treat Saxon genitive morphology as realization on D; older non-DP analyses placed possessors inside NP instead. | Standard DP-Hypothesis treatment; historically controversial against pre-DP NP analyses. | `ABN`, `MIT`, `CAR` |
| Pronouns as DPs | `DP -> D` | Pronouns are treated as full DPs, often as determiner-like heads without an overt NP complement. | Some accounts instead posit a null NP complement; older pre-DP approaches treated pronouns as NPs. | Framework-sensitive detail inside the broader DP Hypothesis. | `MIT`, `ABN`, `CAR` |
| Proper names as DPs | `DP -> D NP[Name]` or `DP -> D` | Proper names are commonly treated as DPs, either with a null D over NP or with the name occupying D. | There is no single universally adopted implementation; DP status is common, exact internal rule varies. | Controversial in detail, but DP status is widespread in DP-based syntax. | `ABN`, `MIT`, `CAR` |
| NP (core under DP) | `NP -> N'`<br>`N' -> N (PP/CP)` | NP is the nominal core below D; complements attach closest to N. | Under the DP Hypothesis, NP no longer hosts determiners as specifiers. Older X-bar / pre-DP grammar often used NP as the maximal nominal phrase. | Canonical DP-compatible NP rule. | `ABN`, `CAR`, `WISC`, `CGEL`, `PTB` |
| NP (adjunction / postmodification) | `N' -> N' PP`<br>`N' -> N' CP`<br>`N' -> AP N'` | NP-internal adjuncts and postmodifiers attach to `N'`, while complements remain sister to N. | Linear order differs by modifier type: English adjectival premodifiers are often shown as left-attaching, while PP and relative-clause postmodifiers are right-attaching. | Core X-bar with category-sensitive linearization. | `CAR`, `WISC`, `CGEL`, `PTB`, `CAM` |
| PP (baseline) | `PP -> P'`<br>`P' -> P DP` | P heads the phrase and selects a DP complement. | Older sources often say NP complement; under a DP-first model, the nominal complement is more properly a DP. | Core X-bar with DP-adjusted complement category. | `CAR`, `WISC`, `CGEL`, `CAM` |
| PP (clausal complement variation) | `P' -> P CP`<br>`P' -> P NonFiniteClause` | Some prepositions select clausal complements rather than nominal ones. | Framework-sensitive in label choice: some sources keep the complement simply as a clause, others distinguish gerund-participial or non-finite clausal projections. | Attested English variation; exact category labels differ by framework. | `CGEL`, `CAM`, `CAR` |
| AP (baseline) | `AP -> (AdvP) A'`<br>`A' -> A (PP/CP)` | Degree or intensifier material occupies specifier-like or left-adjoined position; complements attach closest to A. | Some analyses prefer a separate `DegP` above AP rather than treating intensifiers as Spec,AP or adjuncts. | Core X-bar AP with a well-known degree/split variation. | `CAR`, `WISC`, `CGEL`, `CAM` |
| AP (adjunction) | `A' -> AdvP A'`<br>`A' -> A' PP` | Degree/intensifier material and some secondary modifiers attach outside the head-complement relation. | Exactly where degree material lives is one of the clearest sources of theoretical variation. | Framework-dependent in detail, but X-bar-compatible. | `CAR`, `WISC`, `CAM` |
| AdvP (baseline) | `AdvP -> (AdvP) Adv'`<br>`Adv' -> Adv (PP/CP)` | Adverb phrases project from an adverb head, optionally with intensifier-like material in the outer layer. | Many analyses prefer a separate degree projection over AdvP. | X-bar-compatible, but often split in later work. | `CAR`, `WISC`, `CAM` |
| AdvP (adjunction) | `Adv' -> AdvP Adv'`<br>`Adv' -> Adv' PP` | Degree/intensifier material and some complements/adjuncts attach at `Adv'`. | Far less standardized across sources than DP, TP, or VP rules. | Weakly standardized; category inventory varies by source. | `WISC`, `CAM`, `CAR` |

## Major points of variation across sources

### 1. Whether TP is simple or split

Conservative textbook X-bar syntax often uses a simple clause spine: `CP > TP > VP`. More articulated analyses introduce `vP` beneath T, and some frameworks further split the inflectional domain into projections such as `AgrSP`, `NegP`, and `AspP`. The table records these as variants rather than treating one as silently canonical.

### 2. Whether a vP shell is assumed

If a `vP` shell is assumed, the subject is introduced in `Spec,vP` and T selects `vP` rather than `VP` directly. If not, many textbook presentations keep the simpler `TP -> DP T'; T' -> T VP`. This is one of the biggest theory-sensitive differences in clause architecture.

### 3. The status of pronouns and proper names

Under the DP Hypothesis, pronouns and proper names are commonly treated as DPs rather than NPs, but the internal rule varies:

- pronouns as bare D heads
- pronouns with null NP complement
- proper names as NP under null D
- proper names as occupying D directly

The broad DP status is now familiar in generative syntax, but the exact micro-analysis is not fully uniform.

### 4. Possessor placement in Spec,DP

A standard DP-Hypothesis move is to place possessors in `Spec,DP`: `DP -> DP D'; D' -> D NP`. This sharply contrasts with older NP-centered analyses in which possessors were built inside NP.

### 5. Adjunction vs. complement structure

All sources agree that complements are structurally closer to the head than adjuncts. The disagreement is usually not whether the distinction exists, but rather:

- which dependents count as complements vs adjuncts
- what linear order is assumed for English
- whether later minimalist work should still be stated with rewrite rules at all

### 6. Whether degree material belongs in Spec/AP and Spec/AdvP or in a separate DegP

A common textbook X-bar treatment keeps intensifiers in a specifier-like or left-adjoined position within AP/AdvP. More articulated analyses project a separate degree phrase. Since this document is restricted to the requested categories, the table keeps AP and AdvP rules in X-bar form and marks `DegP` analyses as a point of variation.

### 7. Treebank labels vs. theoretical labels

Penn Treebank materials still use `NP` rather than `DP` for nominal phrases. That remains useful for corpus comparison and testing, but it should not be mistaken for a rejection of the DP Hypothesis.

## Brief consolidated note on the largest disagreements

Across the sources consulted, the most stable rules are the general X-bar schema, the basic `CP -> (Spec) C'; C' -> C TP` clause rule, the `TP -> DP T'` rule, the canonical `DP -> (DP) D'; D' -> D NP` rule under the DP Hypothesis, and the baseline headed projections for VP, PP, AP, and AdvP.

The major disagreements concern:

- whether the clause spine stops at TP or splits into additional projections
- whether a `vP` shell is required
- how pronouns and proper names are represented internally as DPs
- whether AP and AdvP should be split by a dedicated degree projection
- how much detail to encode for adjunct placement and attachment height

## Bibliography of sources consulted

- Abney, Steven. 1987. *The English Noun Phrase in Its Sentential Aspect*. Doctoral dissertation, MIT. Metadata and mirrors consulted at MITWPL, CORE, and Glottolog.
- Cambridge Grammar Today pages consulted: noun phrases; noun phrases and verbs; adjective phrases; adjective phrase functions; adjective phrase position; verb phrases; prepositional phrases; prepositions and particles; adverb phrases; clause types. <https://dictionary.cambridge.org/us/grammar/british-grammar>
- Carnie, Andrew. 2021. *Syntax: A Generative Introduction*. 5th ed. Wiley-Blackwell. Related chapter/workbook pages consulted via OUP, Carnie's site, and Wiley.
- Debnath, Devi Prasanna. “The X-bar Schema.” e-PG Pathshala / INFLIBNET lecture materials. <https://ebooks.inflibnet.ac.in/engp10/chapter/the-x-bar-schema/>
- Haegeman, Liliane. 1994. *Introduction to Government and Binding Theory*. 2nd ed. Blackwell. Publisher metadata consulted via Wiley.
- Huddleston, Rodney, and Geoffrey K. Pullum. 2002. *The Cambridge Grammar of the English Language*. Bibliographic record consulted via Grambank.
- MIT OpenCourseWare / MIT syntax lecture resources consulted for clause structure and pronouns in DP. Main course family consulted: <https://ocw.mit.edu/courses/24-902-language-and-its-structure-ii-syntax-spring-2022/>
- Marcus, Mitchell P., Beatrice Santorini, and Mary Ann Marcinkiewicz. 1993. “Building a Large Annotated Corpus of English: The Penn Treebank.” <https://aclanthology.org/J93-2004/>
- Penn Treebank II bracketing guidelines consulted at <https://repository.upenn.edu/handle/20.500.14332/6998>
- Quirk, Randolph, Sidney Greenbaum, Geoffrey Leech, and Jan Svartvik. 1985. *A Comprehensive Grammar of the English Language*. Open Library record consulted.
- Radford, Andrew. 2004. *Minimalist Syntax: Exploring the Structure of English*. Cambridge University Press. Preview and metadata consulted for split projections and `vP`-shell discussions.
- SDSU lecture notes consulted: “Complements, Specifiers, and Adjuncts.” <https://gawron.sdsu.edu/syntax/new_midterm/adj_comp_spec.htm>
- UBC lecture notes consulted: “X-bar Theory.” <https://wiki.ubc.ca/Course%3ALING300/X-bar_Theory>
- University of Wisconsin phrase-structure slides consulted: “General structure of NP, AdjP, AdvP, and PP.” <https://dept.english.wisc.edu/rfyoung/324/NPAdjPAdvPandPP.pdf>
