use super::permission::Kind as PermissionKind;

use crate::glob::analyzer::has_pattern;

use glob::Pattern;
use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(Pattern),
    Exact(u16),
    Any,
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

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Invalid port: {0}")]
    Parse(#[from] ParseIntError),
    #[error("Invalid glob pattern: {0}")]
    Glob(#[from] glob::PatternError),
}

impl TryFrom<String> for Kind {
    type Error = ErrorKind;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if has_pattern(&value) {
            Ok(Self::Glob(Pattern::new(&value)?))
        } else {
            Ok(Self::Exact(value.parse()?))
        }
    }
}

impl From<u16> for Kind {
    fn from(value: u16) -> Self {
        Self::Exact(value)
    }
}
