use super::permission::Kind as PermissionKind;

use crate::glob::analyzer::has_pattern;

use url::{Host, ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(String),
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

impl TryFrom<String> for Kind {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if has_pattern(&value) {
            Ok(Self::Glob(value))
        } else {
            Ok(Self::Exact(Host::parse(&value)?))
        }
    }
}
