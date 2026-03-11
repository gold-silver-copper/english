use crate::*;

///The Adj struct is used for holding adjective functions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adj {}

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
    /// assert_eq!(Adj::comparative("fast"), "faster");
    /// assert_eq!(Adj::comparative("fun"), "more fun");
    /// ```
    pub fn comparative(word: &str) -> String {
        English::adj(word, &Degree::Comparative)
    }

    /// Returns the superlative form of an adjective.
    ///
    /// # Examples
    /// ```
    /// use english::Adj;
    ///
    /// assert_eq!(Adj::superlative("fast"), "fastest");
    /// assert_eq!(Adj::superlative("fun"), "most fun");
    /// ```
    pub fn superlative(word: &str) -> String {
        English::adj(word, &Degree::Superlative)
    }

    /// Returns the positive (base) form of an adjective.
    ///
    /// # Examples
    /// ```
    /// use english::Adj;
    ///
    /// assert_eq!(Adj::positive("fast"), "fast");
    /// ```
    pub fn positive(word: &str) -> String {
        English::adj(word, &Degree::Positive)
    }
}
