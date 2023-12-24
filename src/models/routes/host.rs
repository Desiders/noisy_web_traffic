use super::permission::Kind as PermissionKind;

use glob::{Pattern, PatternError};
use std::fmt::{self, Display, Formatter};
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

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glob(pattern) => pattern.fmt(f),
            Self::Exact(exact) => exact.fmt(f),
            Self::Any => "*".fmt(f),
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
        let host = Kind::exact("example.com").unwrap();

        assert!(host.matches("example.com"));
        assert!(!host.matches("example.org"));
        assert!(!host.matches("example"));

        let host = Kind::glob("*.example.com").unwrap();

        assert!(host.matches("www.example.com"));
        assert!(host.matches("api.example.com"));
        assert!(host.matches(".example.com"));
        assert!(!host.matches("example.com"));
        assert!(!host.matches("example.org"));
        assert!(!host.matches("example"));

        let host = Kind::glob("*.example.*").unwrap();

        assert!(host.matches("www.example.com"));
        assert!(host.matches("www.example.org"));
        assert!(host.matches("api.example.com"));
        assert!(host.matches("api.example.org"));
        assert!(host.matches(".example.com"));
        assert!(!host.matches("example.com"));
        assert!(!host.matches("example.org"));
        assert!(!host.matches("example"));

        let host = Kind::Any;

        assert!(host.matches("example.com"));
        assert!(host.matches("example.org"));
        assert!(host.matches("example"));

        let host = Kind::glob("ex?mple.com").unwrap();

        assert!(host.matches("example.com"));
        assert!(host.matches("exbmple.com"));
        assert!(host.matches("excmple.com"));
        assert!(!host.matches("exmple.com"));
        assert!(!host.matches("example.org"));
        assert!(!host.matches("example"));

        let host = Kind::glob("ex[ab]mple.com").unwrap();

        assert!(host.matches("example.com"));
        assert!(host.matches("exbmple.com"));
        assert!(!host.matches("excmple.com"));
        assert!(!host.matches("exmple.com"));
        assert!(!host.matches("example.org"));
        assert!(!host.matches("example"));

        let host = Kind::glob("ex[!ab]mple.com").unwrap();

        assert!(!host.matches("example.com"));
        assert!(!host.matches("exbmple.com"));
        assert!(host.matches("excmple.com"));
        assert!(!host.matches("exmple.com"));
        assert!(!host.matches("example.org"));
        assert!(!host.matches("example"));
    }
}
