#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirections {
    pub acceptable: bool,
    pub max_redirects: u16,
}

impl Redirections {
    pub const fn new(acceptable: bool, max_redirects: u16) -> Self {
        Self {
            acceptable,
            max_redirects,
        }
    }
}

impl Default for Redirections {
    fn default() -> Self {
        Self::new(true, 5)
    }
}
