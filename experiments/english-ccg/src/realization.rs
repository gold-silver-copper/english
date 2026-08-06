use english::English;

use crate::derivation::Ccg;
use crate::morphology::{
    default_agreement, override_agreement, realize_token, Agreement, EnglishMorphology,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealizeOpts {
    pub capitalize: bool,
    pub punctuation: Option<char>,
    pub agreement: Agreement,
}

impl RealizeOpts {
    pub fn sentence() -> Self {
        Self {
            capitalize: true,
            punctuation: Some('.'),
            agreement: Agreement::Auto,
        }
    }

    pub fn question() -> Self {
        Self {
            capitalize: true,
            punctuation: Some('?'),
            agreement: Agreement::Auto,
        }
    }

    pub fn fragment() -> Self {
        Self {
            capitalize: false,
            punctuation: None,
            agreement: Agreement::Auto,
        }
    }
}

pub fn realize(ccg: &Ccg) -> String {
    realize_as(ccg, RealizeOpts::fragment())
}

pub fn realize_as(ccg: &Ccg, opts: RealizeOpts) -> String {
    if ccg.reduced_parse().is_none() {
        panic!("cannot realize an unreduced CCG derivation");
    }

    let morphology = EnglishMorphology;
    let subject_range = ccg.subject_range();
    let agreement = override_agreement(opts.agreement)
        .or_else(|| ccg.subject_agreement())
        .unwrap_or_else(default_agreement);

    let mut text = ccg
        .flattened_tokens()
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            realize_token(token, agreement, subject_range.as_ref(), index, &morphology)
        })
        .collect::<Vec<_>>()
        .join(" ");

    if opts.capitalize {
        text = English::capitalize_first(&text);
    }

    if let Some(mark) = opts.punctuation {
        text.push(mark);
    }

    text
}
