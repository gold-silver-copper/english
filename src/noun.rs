use crate::*;

///The Noun struct is used for handling more complicated noun phrases
/// It is interchangeable with strings for all noun functions such as count_with_number()
///
/// # Examples
/// ```
///  let jeans = Noun::from("pair").with_complement("of jeans");
///  assert_eq!(English::count_with_number(jeans, 3), "3 pairs of jeans");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noun {
    pub head: String,
    pub modifier: Option<String>,   // words before the head
    pub complement: Option<String>, // words after the head
}

impl Noun {
    pub fn new(head: impl Into<String>) -> Self {
        Noun {
            head: head.into(),
            modifier: None,
            complement: None,
        }
    }

    pub fn with_specifier(mut self, pre: impl Into<String>) -> Self {
        self.modifier = Some(pre.into());
        self
    }

    pub fn with_complement(mut self, post: impl Into<String>) -> Self {
        self.complement = Some(post.into());
        self
    }
}

impl Noun {
    /// Returns a noun inflected according to the count. Wrapper around English::noun()
    ///
    /// # Examples
    /// ```rust
    /// assert_eq!(English::count("cat", 1), "cat");
    /// assert_eq!(English::count("cat", 2), "cats");
    /// ```
    pub fn count<T: Into<Noun>>(word: T, count: u32) -> String {
        if count == 1 {
            English::noun(word, &Number::Singular)
        } else {
            English::noun(word, &Number::Plural)
        }
    }

    /// Returns a noun inflected according to the count, preserves the number in output
    ///
    /// # Examples
    /// ```rust
    /// assert_eq!(English::count("cat", 1), "1 cat");
    /// assert_eq!(English::count("cat", 2), "2 cats");
    /// ```
    pub fn count_with_number<T: Into<Noun>>(word: T, count: u32) -> String {
        format!("{} {}", count, Noun::count(word, count))
    }

    /// Returns the plural form of a noun.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(English::plural("child"), "children");
    /// assert_eq!(English::plural("cat"), "cats");
    /// ```
    pub fn plural<T: Into<Noun>>(word: T) -> String {
        English::noun(word, &Number::Plural)
    }

    /// Returns the singular form of a noun.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(English::singular("cat2"), "cat");
    /// ```
    pub fn singular<T: Into<Noun>>(word: T) -> String {
        English::noun(word, &Number::Singular)
    }
}

impl From<String> for Noun {
    fn from(s: String) -> Self {
        Noun {
            head: s,
            modifier: None,
            complement: None,
        }
    }
}
impl From<&String> for Noun {
    fn from(s: &String) -> Self {
        Noun {
            head: s.clone(),
            modifier: None,
            complement: None,
        }
    }
}

impl From<&str> for Noun {
    fn from(s: &str) -> Self {
        Noun {
            head: s.to_string(),
            modifier: None,
            complement: None,
        }
    }
}
impl From<&Noun> for Noun {
    fn from(s: &Noun) -> Self {
        s.clone()
    }
}
