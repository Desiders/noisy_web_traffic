use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proxy {
    pub value: String,
}

impl Proxy {
    pub const fn new(value: String) -> Self {
        Self { value }
    }
}

impl Deref for Proxy {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
