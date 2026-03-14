use crate::*;

/// The Noun struct is a lightweight noun lemma wrapper.
/// It is interchangeable with strings for all noun functions such as `count_with_number()`.
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
    /// Returns a noun inflected according to the count. Wrapper around English::noun()
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
            English::noun(self, &Number::Singular)
        } else {
            English::noun(self, &Number::Plural)
        }
    }

    /// Returns a noun inflected according to the count, preserves the number in output
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
        English::noun(self, &Number::Plural)
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
        English::noun(self, &Number::Singular)
    }
}

impl From<String> for Noun {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&String> for Noun {
    fn from(s: &String) -> Self {
        Self(s.clone())
    }
}

impl From<&str> for Noun {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for Noun {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
