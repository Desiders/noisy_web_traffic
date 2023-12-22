use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAgent {
    pub value: String,
}

impl UserAgent {
    pub const fn new(value: String) -> Self {
        Self { value }
    }
}

impl Deref for UserAgent {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
