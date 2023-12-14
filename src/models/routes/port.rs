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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let port = Kind::exact(80);

        assert!(port.matches(80));
        assert!(port.matches_str("80"));
        assert!(!port.matches(8080));
        assert!(!port.matches_str("8080"));

        let port = Kind::exact_str("80").unwrap();

        assert!(port.matches(80));
        assert!(port.matches_str("80"));
        assert!(!port.matches(8080));
        assert!(!port.matches_str("8080"));

        let port = Kind::glob("8?8?").unwrap();

        assert!(port.matches(8080));
        assert!(port.matches_str("8080"));
        assert!(port.matches(8181));
        assert!(port.matches_str("8181"));
        assert!(!port.matches(80));
        assert!(!port.matches_str("80"));
        assert!(!port.matches(8071));
        assert!(!port.matches_str("8071"));

        let port = Kind::glob("1*1*").unwrap();

        assert!(port.matches(1010));
        assert!(port.matches_str("1010"));
        assert!(port.matches(1111));
        assert!(port.matches_str("1111"));
        assert!(port.matches(10010));
        assert!(port.matches_str("10010"));
        assert!(!port.matches(80));
        assert!(!port.matches_str("80"));

        let port = Kind::glob("80*").unwrap();

        assert!(port.matches(8080));
        assert!(port.matches_str("8080"));
        assert!(port.matches(8081));
        assert!(port.matches_str("8081"));
        assert!(port.matches(80));
        assert!(port.matches_str("80"));
        assert!(port.matches(808));
        assert!(port.matches_str("808"));
        assert!(!port.matches(11));
        assert!(!port.matches_str("11"));
    }
}
