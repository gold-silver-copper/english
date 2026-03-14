use crate::*;

/// A lightweight wrapper around a noun lemma.
///
/// Use [`English::noun`] for low-level `&str` inflection, or the methods on
/// `Noun` for a more ergonomic typed API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noun(String);

impl Noun {
    /// Creates a new noun with the given head.
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

impl Noun {
    /// Inflects the noun according to the given count.
    ///
    /// # Examples
    /// ```rust
    /// use english::Noun;
    ///
    /// assert_eq!(Noun::new("cat").count(1), "cat");
    /// assert_eq!(Noun::new("cat").count(2), "cats");
    /// ```
    pub fn count(&self, count: u32) -> String {
        if count == 1 {
            English::noun(self.as_str(), &Number::Singular)
        } else {
            English::noun(self.as_str(), &Number::Plural)
        }
    }

    /// Inflects the noun according to the given count and keeps the number in the output.
    ///
    /// # Examples
    /// ```rust
    /// use english::Noun;
    ///
    /// assert_eq!(Noun::new("cat").count_with_number(1), "1 cat");
    /// assert_eq!(Noun::new("cat").count_with_number(2), "2 cats");
    /// ```
    pub fn count_with_number(&self, count: u32) -> String {
        format!("{} {}", count, self.count(count))
    }

    /// Returns the plural form of a noun.
    ///
    /// # Examples
    /// ```
    /// use english::Noun;
    ///
    /// assert_eq!(Noun::new("child").plural(), "children");
    /// assert_eq!(Noun::new("cat").plural(), "cats");
    /// ```
    pub fn plural(&self) -> String {
        English::noun(self.as_str(), &Number::Plural)
    }

    /// Returns the singular form of a noun.
    ///
    /// # Examples
    /// ```
    /// use english::Noun;
    ///
    /// assert_eq!(Noun::new("cat2").singular(), "cat");
    /// ```
    pub fn singular(&self) -> String {
        English::noun(self.as_str(), &Number::Singular)
    }
}

impl From<String> for Noun {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Noun {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
