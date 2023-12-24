use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAgent {
    pub value: String,
}

impl UserAgent {
    pub const fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for UserAgent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.value)
    }
}

impl Deref for UserAgent {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<str> for UserAgent {
    fn as_ref(&self) -> &str {
        &self.value
    }
}
