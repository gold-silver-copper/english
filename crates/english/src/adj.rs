use crate::*;

/// The Adj struct is a lightweight adjective lemma wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adj(String);

impl Adj {
    /// Creates a new adjective with the given head.
    pub fn new(head: impl Into<String>) -> Self {
        Self(head.into())
    }

    /// Borrows the underlying lemma.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the underlying lemma.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Adj {
    // ---------------------------
    // ADJECTIVE HELPERS
    // ---------------------------

    /// Returns the comparative form of an adjective.
    ///
    /// # Examples
    /// ```
    /// use english::Adj;
    ///
    /// assert_eq!(Adj::new("fast").comparative(), "faster");
    /// assert_eq!(Adj::new("fun").comparative(), "more fun");
    /// ```
    pub fn comparative(&self) -> String {
        English::adj(self.as_str(), &Degree::Comparative)
    }

    /// Returns the superlative form of an adjective.
    ///
    /// # Examples
    /// ```
    /// use english::Adj;
    ///
    /// assert_eq!(Adj::new("fast").superlative(), "fastest");
    /// assert_eq!(Adj::new("fun").superlative(), "most fun");
    /// ```
    pub fn superlative(&self) -> String {
        English::adj(self.as_str(), &Degree::Superlative)
    }

    /// Returns the positive (base) form of an adjective.
    ///
    /// # Examples
    /// ```
    /// use english::Adj;
    ///
    /// assert_eq!(Adj::new("fast").positive(), "fast");
    /// ```
    pub fn positive(&self) -> String {
        English::adj(self.as_str(), &Degree::Positive)
    }
}

impl From<String> for Adj {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Adj {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
