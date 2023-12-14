use super::permission::Kind as PermissionKind;

use glob::{Pattern, PatternError};
use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(Pattern),
    Exact(u16),
    Any,
}

impl Kind {
    pub fn glob(pattern: impl AsRef<str>) -> Result<Self, PatternError> {
        Ok(Self::Glob(Pattern::new(pattern.as_ref())?))
    }

    pub const fn exact(port: u16) -> Self {
        Self::Exact(port)
    }

    pub fn exact_str(port: impl AsRef<str>) -> Result<Self, ParseIntError> {
        Ok(Self::Exact(port.as_ref().parse()?))
    }

    pub fn matches(&self, port: u16) -> bool {
        match self {
            Self::Glob(pattern) => pattern.matches(&port.to_string()),
            Self::Exact(exact) => exact == &port,
            Self::Any => true,
        }
    }

    pub fn matches_str(&self, port: impl AsRef<str>) -> bool {
        match self {
            Self::Glob(pattern) => pattern.matches(port.as_ref()),
            Self::Exact(exact) => exact.to_string() == port.as_ref(),
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
