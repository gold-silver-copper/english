use crate::*;

/// The Verb struct is a lightweight verb lemma wrapper.
/// It is interchangeable with strings for all verb inflection helpers such as `present_participle()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb(String);

impl Verb {
    /// Create a new verb with just the head.
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

impl Verb {
    /// Returns the third-person singular present tense of the verb.
    ///
    /// # Examples
    /// ```
    /// use english::Verb;
    ///
    /// assert_eq!(Verb::new("run").third_person(), "runs");
    /// ```
    pub fn third_person(&self) -> String {
        English::verb(
            self.as_str(),
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
        )
    }

    /// Returns the past tense of the verb.
    ///
    /// # Examples
    /// ```
    /// use english::Verb;
    ///
    /// assert_eq!(Verb::new("walk").past(), "walked");
    /// ```
    pub fn past(&self) -> String {
        English::verb(
            self.as_str(),
            &Person::Third,    // person doesn’t matter in past tense finite
            &Number::Singular, // irrelevant
            &Tense::Past,
            &Form::Finite,
        )
    }

    /// Returns the present participle ("-ing" form) of the verb.
    ///
    /// # Examples
    /// ```
    /// use english::Verb;
    ///
    /// assert_eq!(Verb::new("swim").present_participle(), "swimming");
    /// ```
    pub fn present_participle(&self) -> String {
        English::verb(
            self.as_str(),
            &Person::First,    // irrelevant for participles
            &Number::Singular, // irrelevant
            &Tense::Present,
            &Form::Participle,
        )
    }

    /// Returns the past participle of the verb.
    ///
    /// # Examples
    /// ```
    /// use english::Verb;
    ///
    /// assert_eq!(Verb::new("eat").past_participle(), "eaten");
    /// ```
    pub fn past_participle(&self) -> String {
        English::verb(
            self.as_str(),
            &Person::First,    // irrelevant
            &Number::Singular, // irrelevant
            &Tense::Past,
            &Form::Participle,
        )
    }

    /// Returns the infinitive (base) form of the verb.
    ///
    /// # Examples
    /// ```
    /// use english::Verb;
    ///
    /// assert_eq!(Verb::new("lie2").infinitive(), "lie");
    /// ```
    pub fn infinitive(&self) -> String {
        English::verb(
            self.as_str(),
            &Person::First,    // irrelevant
            &Number::Singular, // irrelevant
            &Tense::Present,   // irrelevant
            &Form::Infinitive,
        )
    }
}

impl From<String> for Verb {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Verb {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
