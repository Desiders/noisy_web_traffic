use super::permission::Kind as PermissionKind;

use crate::glob::analyzer::has_pattern;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(String),
    Exact(String),
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

impl From<String> for Kind {
    fn from(value: String) -> Self {
        if has_pattern(&value) {
            Self::Glob(value)
        } else {
            Self::Exact(value)
        }
    }
}
