use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowRobotsExclusionProtocol {
    pub value: bool,
}

impl FollowRobotsExclusionProtocol {
    pub const fn new(value: bool) -> Self {
        Self { value }
    }
}

impl Default for FollowRobotsExclusionProtocol {
    fn default() -> Self {
        Self::new(true)
    }
}

impl Display for FollowRobotsExclusionProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl Deref for FollowRobotsExclusionProtocol {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
