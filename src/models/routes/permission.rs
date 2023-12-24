use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Acceptable,
    Unacceptable,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Acceptable => "acceptable".fmt(f),
            Kind::Unacceptable => "unacceptable".fmt(f),
        }
    }
}
