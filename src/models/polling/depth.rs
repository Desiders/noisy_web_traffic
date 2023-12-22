#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Depth {
    pub acceptable: bool,
    pub max_depth: u16,
}

impl Depth {
    pub const fn new(acceptable: bool, max_depth: u16) -> Self {
        Self {
            acceptable,
            max_depth,
        }
    }

    pub fn matches(&self, depth: u16) -> bool {
        if self.acceptable {
            depth < self.max_depth
        } else {
            false
        }
    }
}

impl Default for Depth {
    fn default() -> Self {
        Self::new(true, 7)
    }
}
