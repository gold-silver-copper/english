use crate::*;

///The Verb struct is used for handling more complicated verb phrases
/// It is interchangeable with strings for all verb functions such as present_participle()
///
/// # Examples
/// ```
///  let pick_up = Verb::from("pick").with_particle("up");
///  assert_eq!(English::past_participle(pick_up), "picked up");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb {
    pub head: String,             // "pick"
    pub particle: Option<String>, // "up"
}

impl Verb {
    /// Create a new verb with just the head.
    pub fn new(head: impl Into<String>) -> Self {
        Verb {
            head: head.into(),
            particle: None,
        }
    }

    /// Set the particle of a phrasal verb.
    /// # Examples
    /// ```
    ///  let pick_up = Verb::from("pick").with_particle("up");
    ///  assert_eq!(English::past_participle(pick_up), "picked up");
    /// ```
    pub fn with_particle(mut self, particle: impl Into<String>) -> Self {
        self.particle = Some(particle.into());
        self
    }
}

impl Verb {
    /// Returns the third-person singular present tense of the verb.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(English::third_person("run"), "runs");
    /// ```
    pub fn third_person<T: Into<Verb>>(wordish: T) -> String {
        English::verb(
            wordish,
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
    /// assert_eq!(English::past("walk"), "walked");
    /// ```
    pub fn past<T: Into<Verb>>(wordish: T) -> String {
        English::verb(
            wordish,
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
    /// assert_eq!(English::present_participle("swim"), "swimming");
    /// ```
    pub fn present_participle<T: Into<Verb>>(wordish: T) -> String {
        English::verb(
            wordish,
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
    /// assert_eq!(English::past_participle("eat"), "eaten");
    /// ```
    pub fn past_participle<T: Into<Verb>>(wordish: T) -> String {
        English::verb(
            wordish,
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
    /// assert_eq!(English::infinitive("lie2"), "lie");
    /// ```
    pub fn infinitive<T: Into<Verb>>(wordish: T) -> String {
        English::verb(
            wordish,
            &Person::First,    // irrelevant
            &Number::Singular, // irrelevant
            &Tense::Present,   // irrelevant
            &Form::Infinitive,
        )
    }
}

impl From<String> for Verb {
    fn from(s: String) -> Self {
        Verb {
            head: s,
            particle: None,
        }
    }
}

impl From<&String> for Verb {
    fn from(s: &String) -> Self {
        Verb {
            head: s.clone(),
            particle: None,
        }
    }
}

impl From<&str> for Verb {
    fn from(s: &str) -> Self {
        Verb {
            head: s.to_string(),
            particle: None,
        }
    }
}

/// Just clones it
impl From<&Verb> for Verb {
    fn from(s: &Verb) -> Self {
        s.clone()
    }
}
