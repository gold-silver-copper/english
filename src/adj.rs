use crate::*;

impl English {
    // ---------------------------
    // ADJECTIVE HELPERS
    // ---------------------------

    /// Returns the comparative form of an adjective.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(English::comparative("fast2"), "faster");
    /// assert_eq!(English::comparative("fun"), "more fun");
    /// ```
    pub fn comparative(word: &str) -> String {
        English::adj(word, &Degree::Comparative)
    }

    /// Returns the superlative form of an adjective.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(English::superlative("fast2"), "fastest");
    /// assert_eq!(English::superlative("fun"), "most fun");
    /// ```
    pub fn superlative(word: &str) -> String {
        English::adj(word, &Degree::Superlative)
    }

    /// Returns the positive (base) form of an adjective.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(English::positive("fast2"), "fast");
    /// ```
    pub fn positive(word: &str) -> String {
        English::adj(word, &Degree::Positive)
    }
}
