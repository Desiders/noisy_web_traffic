use super::permission::Kind as PermissionKind;

use glob::{Pattern, PatternError};
use url::{Host, ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(Pattern),
    Exact(Host),
    Any,
}

impl Kind {
    pub fn glob(pattern: impl AsRef<str>) -> Result<Self, PatternError> {
        Ok(Self::Glob(Pattern::new(pattern.as_ref())?))
    }

    pub fn exact(host: impl AsRef<str>) -> Result<Self, ParseError> {
        Ok(Self::Exact(Host::parse(host.as_ref())?))
    }

    pub fn matches(&self, host: impl AsRef<str>) -> bool {
        match self {
            Self::Glob(pattern) => pattern.matches(host.as_ref()),
            Self::Exact(exact) => exact.to_string() == host.as_ref(),
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
