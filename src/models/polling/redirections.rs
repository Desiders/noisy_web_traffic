use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirections {
    acceptable: bool,
    max_redirects: u16,
}

impl Redirections {
    pub const fn new(acceptable: bool, max_redirects: u16) -> Self {
        Self {
            acceptable,
            max_redirects,
        }
    }

    #[allow(dead_code)]
    pub const fn acceptable(&self) -> bool {
        self.acceptable
    }

    pub const fn max_redirects(&self) -> u16 {
        if self.acceptable {
            self.max_redirects
        } else {
            0
        }
    }
}

impl Display for Redirections {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.acceptable {
            write!(f, "acceptable redirections: {}", self.max_redirects)
        } else {
            write!(f, "unacceptable redirections")
        }
    }
}

impl Default for Redirections {
    fn default() -> Self {
        Self::new(true, 5)
    }
}
