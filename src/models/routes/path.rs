use super::permission::Kind as PermissionKind;

use glob::{Pattern, PatternError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(Pattern),
    Exact(String),
    Any,
}

impl Kind {
    pub fn glob(pattern: impl AsRef<str>) -> Result<Self, PatternError> {
        Ok(Self::Glob(Pattern::new(pattern.as_ref())?))
    }

    pub fn exact(path: impl Into<String>) -> Self {
        Self::Exact(path.into())
    }

    pub fn matches(&self, path: impl AsRef<str>) -> bool {
        match self {
            Self::Glob(pattern) => pattern.matches(path.as_ref()),
            Self::Exact(exact) => exact == path.as_ref(),
            Self::Any => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Matcher {
    pub permission: PermissionKind,
    pub kind: Kind,
}

impl Matcher {
    pub const fn new(permission: PermissionKind, kind: Kind) -> Self {
        Self { permission, kind }
    }
}
