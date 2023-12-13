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

    pub fn exact(host: impl Into<String>) -> Self {
        Self::Exact(host.into())
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
