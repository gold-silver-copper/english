use english::{Gender, Number, Person};

use crate::builders::Conj;
use crate::cat::{can_bapply, can_bcomp, can_fapply, can_fcomp, type_raise, Cat};
use crate::derivation::{
    assemble, opaque_atom, AgreementInfo, Ccg, CombRule, DerivTree, Parse, ParseMeta, TokenKind,
};

pub fn fapply(func: Ccg, arg: Ccg) -> Ccg {
    let left = expect_parse(&func);
    let right = expect_parse(&arg);
    let cat = can_fapply(&left.cat, &right.cat)
        .unwrap_or_else(|| panic!("forward application failed: {} + {}", left.cat, right.cat));
    combine_forced(func, arg, left, right, cat, CombRule::ForwardApplication)
}

pub fn bapply(arg: Ccg, func: Ccg) -> Ccg {
    let left = expect_parse(&arg);
    let right = expect_parse(&func);
    let cat = can_bapply(&left.cat, &right.cat)
        .unwrap_or_else(|| panic!("backward application failed: {} + {}", left.cat, right.cat));
    combine_forced(arg, func, left, right, cat, CombRule::BackwardApplication)
}

pub fn fcomp(f: Ccg, g: Ccg) -> Ccg {
    let left = expect_parse(&f);
    let right = expect_parse(&g);
    let cat = can_fcomp(&left.cat, &right.cat)
        .unwrap_or_else(|| panic!("forward composition failed: {} + {}", left.cat, right.cat));
    combine_forced(f, g, left, right, cat, CombRule::ForwardComposition)
}

pub fn bcomp(f: Ccg, g: Ccg) -> Ccg {
    let left = expect_parse(&f);
    let right = expect_parse(&g);
    let cat = can_bcomp(&left.cat, &right.cat)
        .unwrap_or_else(|| panic!("backward composition failed: {} + {}", left.cat, right.cat));
    combine_forced(f, g, left, right, cat, CombRule::BackwardComposition)
}

pub fn traise(x: Ccg) -> Ccg {
    let parse = expect_parse(&x);
    let cat =
        type_raise(&parse.cat).unwrap_or_else(|| panic!("type raising failed: {}", parse.cat));
    let derivation = DerivTree::Inner {
        cat: cat.clone(),
        rule: CombRule::TypeRaising,
        left: Box::new(parse.derivation.clone()),
        right: Box::new(DerivTree::Gap {
            cat: crate::cat::bwd(Cat::S, Cat::NP),
        }),
    };
    assemble(
        x.atoms,
        Some(Parse {
            cat,
            derivation,
            meta: ParseMeta {
                span: parse.meta.span.clone(),
                np_agreement: None,
                subject_agreement: parse.meta.np_agreement,
                animacy: parse.meta.animacy,
                subject_range: Some(parse.meta.span.clone()),
            },
        }),
    )
}

pub fn coord(conj: Conj, left: Ccg, right: Ccg) -> Ccg {
    let left_parse = expect_parse(&left);
    let right_parse = expect_parse(&right);
    if left_parse.cat != right_parse.cat {
        panic!(
            "coordination requires matching categories, got {} and {}",
            left_parse.cat, right_parse.cat
        );
    }

    let cat = left_parse.cat.clone();
    let conj_token = crate::derivation::Token {
        surface: conj.as_str().to_string(),
        cat: cat.clone(),
        kind: TokenKind::Conj { _conj: conj },
        animacy: None,
        agreement: None,
    };

    let mut tokens = left.flattened_tokens();
    tokens.push(conj_token.clone());
    tokens.extend(right.flattened_tokens());

    let derivation = DerivTree::Inner {
        cat: cat.clone(),
        rule: CombRule::Coordination,
        left: Box::new(left_parse.derivation.clone()),
        right: Box::new(DerivTree::Inner {
            cat: cat.clone(),
            rule: CombRule::Coordination,
            left: Box::new(DerivTree::Leaf {
                surface: conj.as_str().to_string(),
                cat: cat.clone(),
            }),
            right: Box::new(right_parse.derivation.clone()),
        }),
    };

    let meta = if cat == Cat::NP {
        ParseMeta {
            span: 0..tokens.len(),
            np_agreement: Some(AgreementInfo {
                person: Person::Third,
                number: Number::Plural,
                gender: Gender::Neuter,
            }),
            subject_agreement: None,
            animacy: left_parse.meta.animacy.or(right_parse.meta.animacy),
            subject_range: None,
        }
    } else {
        ParseMeta {
            span: 0..tokens.len(),
            np_agreement: None,
            subject_agreement: None,
            animacy: None,
            subject_range: None,
        }
    };

    opaque_atom(cat, derivation, tokens, meta)
}

fn expect_parse(ccg: &Ccg) -> Parse {
    ccg.reduced_parse()
        .cloned()
        .unwrap_or_else(|| panic!("cannot force a combinator on an unreduced derivation"))
}

fn combine_forced(
    left_ccg: Ccg,
    right_ccg: Ccg,
    left: Parse,
    right: Parse,
    cat: Cat,
    rule: CombRule,
) -> Ccg {
    let derivation = DerivTree::Inner {
        cat: cat.clone(),
        rule,
        left: Box::new(left.derivation.clone()),
        right: Box::new(right.derivation.clone()),
    };

    let mut atoms = left_ccg.atoms;
    atoms.extend(right_ccg.atoms);

    let meta = match rule {
        CombRule::ForwardApplication => ParseMeta {
            span: left.meta.span.start..right.meta.span.end,
            np_agreement: if matches!(cat, Cat::NP | Cat::N) {
                right.meta.np_agreement.or(left.meta.np_agreement)
            } else {
                None
            },
            subject_agreement: left.meta.subject_agreement.or(right.meta.subject_agreement),
            animacy: right.meta.animacy.or(left.meta.animacy),
            subject_range: left
                .meta
                .subject_range
                .clone()
                .or(right.meta.subject_range.clone()),
        },
        CombRule::BackwardApplication => ParseMeta {
            span: left.meta.span.start..right.meta.span.end,
            np_agreement: if matches!(cat, Cat::NP | Cat::N) {
                left.meta.np_agreement.or(right.meta.np_agreement)
            } else {
                None
            },
            subject_agreement: if cat == Cat::S && left.meta.np_agreement.is_some() {
                left.meta.np_agreement
            } else {
                right.meta.subject_agreement.or(left.meta.subject_agreement)
            },
            animacy: left.meta.animacy.or(right.meta.animacy),
            subject_range: if cat == Cat::S && left.meta.np_agreement.is_some() {
                Some(left.meta.span.clone())
            } else {
                right
                    .meta
                    .subject_range
                    .clone()
                    .or(left.meta.subject_range.clone())
            },
        },
        CombRule::ForwardComposition | CombRule::BackwardComposition => ParseMeta {
            span: left.meta.span.start..right.meta.span.end,
            np_agreement: None,
            subject_agreement: left.meta.subject_agreement.or(right.meta.subject_agreement),
            animacy: left.meta.animacy.or(right.meta.animacy),
            subject_range: left
                .meta
                .subject_range
                .clone()
                .or(right.meta.subject_range.clone()),
        },
        _ => ParseMeta::default(),
    };

    assemble(
        atoms,
        Some(Parse {
            cat,
            derivation,
            meta,
        }),
    )
}
