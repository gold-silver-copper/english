use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cat {
    S,
    N,
    NP,
    PP,
    Fwd(Box<Cat>, Box<Cat>),
    Bwd(Box<Cat>, Box<Cat>),
}

pub fn fwd(result: Cat, arg: Cat) -> Cat {
    Cat::Fwd(Box::new(result), Box::new(arg))
}

pub fn bwd(result: Cat, arg: Cat) -> Cat {
    Cat::Bwd(Box::new(result), Box::new(arg))
}

impl Cat {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::S | Self::N | Self::NP | Self::PP)
    }

    pub fn result(&self) -> Option<&Cat> {
        match self {
            Self::Fwd(result, _) | Self::Bwd(result, _) => Some(result),
            _ => None,
        }
    }

    pub fn arg(&self) -> Option<&Cat> {
        match self {
            Self::Fwd(_, arg) | Self::Bwd(_, arg) => Some(arg),
            _ => None,
        }
    }

    pub fn to_notation(&self) -> String {
        match self {
            Self::S => "S".to_string(),
            Self::N => "N".to_string(),
            Self::NP => "NP".to_string(),
            Self::PP => "PP".to_string(),
            Self::Fwd(result, arg) => format!("{}/{}", wrap(result), wrap(arg)),
            Self::Bwd(result, arg) => format!("{}\\{}", wrap(result), wrap(arg)),
        }
    }
}

impl fmt::Display for Cat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_notation())
    }
}

fn wrap(cat: &Cat) -> String {
    if cat.is_complete() {
        cat.to_notation()
    } else {
        format!("({})", cat.to_notation())
    }
}

pub fn can_fapply(left: &Cat, right: &Cat) -> Option<Cat> {
    match left {
        Cat::Fwd(result, arg) if arg.as_ref() == right => Some((**result).clone()),
        _ => None,
    }
}

pub fn can_bapply(left: &Cat, right: &Cat) -> Option<Cat> {
    match right {
        Cat::Bwd(result, arg) if arg.as_ref() == left => Some((**result).clone()),
        _ => None,
    }
}

pub fn can_fcomp(left: &Cat, right: &Cat) -> Option<Cat> {
    match (left, right) {
        (Cat::Fwd(x, y1), Cat::Fwd(y2, z)) if y1 == y2 => Some(fwd((**x).clone(), (**z).clone())),
        _ => None,
    }
}

pub fn can_bcomp(left: &Cat, right: &Cat) -> Option<Cat> {
    match (left, right) {
        (Cat::Bwd(y1, z), Cat::Bwd(x, y2)) if y1 == y2 => Some(bwd((**x).clone(), (**z).clone())),
        _ => None,
    }
}

pub fn type_raise(cat: &Cat) -> Option<Cat> {
    match cat {
        Cat::NP => Some(fwd(Cat::S, bwd(Cat::S, Cat::NP))),
        _ => None,
    }
}

pub fn parse_cat(input: &str) -> Result<Cat, String> {
    let compact: String = input.chars().filter(|ch| !ch.is_whitespace()).collect();
    let chars: Vec<char> = compact.chars().collect();
    let mut parser = Parser {
        chars: &chars,
        idx: 0,
    };
    let cat = parser.parse_expr()?;
    if parser.idx != chars.len() {
        return Err(format!("unexpected trailing input in category `{input}`"));
    }
    Ok(cat)
}

struct Parser<'a> {
    chars: &'a [char],
    idx: usize,
}

impl<'a> Parser<'a> {
    fn parse_expr(&mut self) -> Result<Cat, String> {
        let mut lhs = self.parse_primary()?;
        while let Some(op) = self.peek() {
            match op {
                '/' | '\\' => {
                    self.idx += 1;
                    let rhs = self.parse_primary()?;
                    lhs = if op == '/' {
                        fwd(lhs, rhs)
                    } else {
                        bwd(lhs, rhs)
                    };
                }
                ')' => break,
                _ => return Err(format!("unexpected token `{op}` in category")),
            }
        }
        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Cat, String> {
        match self.peek() {
            Some('(') => {
                self.idx += 1;
                let cat = self.parse_expr()?;
                match self.peek() {
                    Some(')') => {
                        self.idx += 1;
                        Ok(cat)
                    }
                    _ => Err("expected `)` in category".to_string()),
                }
            }
            Some('S') => {
                self.idx += 1;
                Ok(Cat::S)
            }
            Some('N') => {
                self.idx += 1;
                if self.peek() == Some('P') {
                    self.idx += 1;
                    Ok(Cat::NP)
                } else {
                    Ok(Cat::N)
                }
            }
            Some('P') => {
                self.idx += 1;
                if self.peek() == Some('P') {
                    self.idx += 1;
                    Ok(Cat::PP)
                } else {
                    Err("expected `PP`".to_string())
                }
            }
            Some(other) => Err(format!("unexpected token `{other}` in category")),
            None => Err("unexpected end of category".to_string()),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.idx).copied()
    }
}

#[macro_export]
macro_rules! cat {
    ($lit:literal) => {
        $crate::cat::parse_cat($lit).expect("invalid category literal")
    };
    (S) => {
        $crate::cat::Cat::S
    };
    (N) => {
        $crate::cat::Cat::N
    };
    (NP) => {
        $crate::cat::Cat::NP
    };
    (PP) => {
        $crate::cat::Cat::PP
    };
}
