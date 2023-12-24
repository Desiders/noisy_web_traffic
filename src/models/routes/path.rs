use super::permission::Kind as PermissionKind;

use glob::{Pattern, PatternError};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Glob(Pattern),
    Exact(String),
    Any,
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

impl Kind {
    pub fn glob(pattern: impl AsRef<str>) -> Result<Self, PatternError> {
        Ok(Self::Glob(Pattern::new(pattern.as_ref())?))
    }

    pub fn exact(path: impl Into<String>) -> Self {
        Self::Exact(path.into())
    }

    pub fn matches(&self, path: impl AsRef<str>) -> bool {
        let path = path.as_ref();

        let path = if path == "/" {
            path
        } else {
            path.strip_suffix('/').unwrap_or(path)
        };

        match self {
            Self::Glob(pattern) => pattern.matches(path),
            Self::Exact(exact) => exact == path,
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
        let path = Kind::exact("/foo/bar");

        assert!(path.matches("/foo/bar"));
        assert!(path.matches("/foo/bar/"));
        assert!(!path.matches("/foo"));
        assert!(!path.matches("/foo/"));
        assert!(!path.matches("/foo/bar/baz"));
        assert!(!path.matches("/foo/bar/baz/"));

        let path = Kind::glob("/foo/*").unwrap();

        assert!(path.matches("/foo/bar"));
        assert!(path.matches("/foo/bar/"));
        assert!(path.matches("/foo/bar/baz"));
        assert!(path.matches("/foo/bar/baz/"));
        assert!(!path.matches("/foo"));
        assert!(!path.matches("/foo/"));

        let path = Kind::glob("/foo/*/baz").unwrap();

        assert!(path.matches("/foo/bar/baz"));
        assert!(path.matches("/foo/bar/baz/"));
        assert!(path.matches("/foo/a/baz"));
        assert!(path.matches("/foo/a/baz/"));
        assert!(!path.matches("/foo/bar"));
        assert!(!path.matches("/foo/bar/"));
        assert!(!path.matches("/foo/bar/bar"));
        assert!(!path.matches("/foo/bar/bar/"));
        assert!(!path.matches("/foo/a/bar"));
        assert!(!path.matches("/foo/a/bar/"));

        let path = Kind::glob("/foo/?/baz").unwrap();

        assert!(path.matches("/foo/a/baz"));
        assert!(path.matches("/foo/a/baz/"));
        assert!(path.matches("/foo/b/baz"));
        assert!(path.matches("/foo/b/baz/"));
        assert!(!path.matches("/foo/bar/baz"));
        assert!(!path.matches("/foo/bar/baz/"));
        assert!(!path.matches("/foo/a/bar"));
        assert!(!path.matches("/foo/a/bar/"));
        assert!(!path.matches("/foo/b/bar"));
        assert!(!path.matches("/foo/b/bar/"));

        let path = Kind::exact("/");

        assert!(path.matches("/"));
        assert!(!path.matches("/foo"));
        assert!(!path.matches("/foo/"));

        let path = Kind::exact("");

        assert!(path.matches(""));
        assert!(!path.matches("/"));
        assert!(!path.matches("/foo"));
        assert!(!path.matches("/foo/"));
    }
}
