#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noun {
    head: String,
    pre_mod: Option<String>,
    post_mod: Option<String>,
}

impl Noun {
    pub fn new(head: impl Into<String>) -> Self {
        Noun {
            head: head.into(),
            pre_mod: None,
            post_mod: None,
        }
    }

    pub fn with_pre_mod(mut self, pre: impl Into<String>) -> Self {
        self.pre_mod = Some(pre.into());
        self
    }

    pub fn with_post_mod(mut self, post: impl Into<String>) -> Self {
        self.post_mod = Some(post.into());
        self
    }

    /// Returns the singular phrase.
    pub fn singular(&self) -> String {
        format!(
            "{}{}{}",
            self.pre_mod
                .as_ref()
                .map(|p| format!("{} ", p))
                .unwrap_or_default(),
            self.head,
            self.post_mod
                .as_ref()
                .map(|p| format!(" {}", p))
                .unwrap_or_default()
        )
    }

    /// Returns the plural phrase.
    pub fn plural(&self) -> String {
        let plural_head = English::noun(&self.head, &Number::Plural);
        format!(
            "{}{}{}",
            self.pre_mod
                .as_ref()
                .map(|p| format!("{} ", p))
                .unwrap_or_default(),
            plural_head,
            self.post_mod
                .as_ref()
                .map(|p| format!(" {}", p))
                .unwrap_or_default()
        )
    }

    /// Returns the noun phrase with the correct number (singular/plural).
    pub fn inflect(&self, number: &Number) -> String {
        match number {
            Number::Singular => self.singular(),
            Number::Plural => self.plural(),
        }
    }
}
