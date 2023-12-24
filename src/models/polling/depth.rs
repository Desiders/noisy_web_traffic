use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Depth {
    acceptable: bool,
    max_depth: u16,
}

impl Depth {
    pub const fn new(acceptable: bool, max_depth: u16) -> Self {
        Self {
            acceptable,
            max_depth,
        }
    }

    pub const fn new_unacceptable() -> Self {
        Self::new(false, 0)
    }

    pub const fn new_acceptable(max_depth: u16) -> Self {
        Self::new(true, max_depth)
    }

    pub const fn acceptable(&self) -> bool {
        self.acceptable
    }

    pub const fn max_depth(&self) -> u16 {
        if self.acceptable {
            self.max_depth
        } else {
            0
        }
    }

    pub const fn matches(&self, depth: u16) -> bool {
        depth < self.max_depth()
    }
}

impl Display for Depth {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.acceptable {
            write!(f, "acceptable depth: {}", self.max_depth)
        } else {
            write!(f, "unacceptable depth")
        }
    }
}

impl Default for Depth {
    fn default() -> Self {
        Self::new(true, 7)
    }
}
