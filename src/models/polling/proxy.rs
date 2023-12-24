use std::{fmt::Display, ops::Deref};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proxy {
    pub value: String,
}

impl Proxy {
    pub const fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.value)
    }
}

impl Deref for Proxy {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<str> for Proxy {
    fn as_ref(&self) -> &str {
        &self.value
    }
}
