#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserAgent {
    pub value: String,
}

impl UserAgent {
    pub const fn new(value: String) -> Self {
        Self { value }
    }
}
