#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proxy {
    pub value: String,
}

impl Proxy {
    pub const fn new(value: String) -> Self {
        Self { value }
    }
}
