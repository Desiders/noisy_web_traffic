use super::permission::Kind as PermissionKind;

use crate::glob::analyzer::has_pattern;

use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(String),
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

impl TryFrom<String> for Kind {
    type Error = ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if has_pattern(&value) {
            Ok(Self::Glob(value))
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
