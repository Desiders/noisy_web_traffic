use super::permission::Kind as PermissionKind;

use crate::glob::analyzer::has_pattern;

use glob::Pattern;
use url::{Host, ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(Pattern),
    Exact(Host),
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
    #[error("Invalid host: {0}")]
    Parse(#[from] ParseError),
    #[error("Invalid glob pattern: {0}")]
    Glob(#[from] glob::PatternError),
}

impl TryFrom<String> for Kind {
    type Error = ErrorKind;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if has_pattern(&value) {
            Ok(Self::Glob(Pattern::new(&value)?))
        } else {
            Ok(Self::Exact(Host::parse(&value)?))
        }
    }
}
