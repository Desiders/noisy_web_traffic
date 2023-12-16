use super::{
    path::{Kind, Matcher},
    permission::Kind as PermissionKind,
};

#[derive(Debug, Default, Clone)]
pub struct Paths {
    pub acceptable: Vec<Kind>,
    pub unacceptable: Vec<Kind>,
}

impl Paths {
    pub fn new(paths: impl IntoIterator<Item = Matcher>) -> Self {
        let mut acceptable = vec![];
        let mut unacceptable = vec![];

        for path in paths {
            match path.permission {
                PermissionKind::Acceptable => acceptable.push(path.kind),
                PermissionKind::Unacceptable => unacceptable.push(path.kind),
            }
        }

        if acceptable.is_empty() {
            acceptable.push(Kind::Any);
        }

        Self {
            acceptable,
            unacceptable,
        }
    }

    pub fn extend(&mut self, paths: impl IntoIterator<Item = Matcher>) {
        for path in paths {
            match path.permission {
                PermissionKind::Acceptable => self.acceptable.push(path.kind),
                PermissionKind::Unacceptable => self.unacceptable.push(path.kind),
            }
        }
    }

    pub fn matches(&self, path: impl AsRef<str>) -> bool {
        let path = path.as_ref();

        let matched_any = self.acceptable.iter().any(|kind| kind.matches(path));

        if !matched_any {
            return false;
        }

        let matched_none = self.unacceptable.iter().any(|kind| kind.matches(path));

        !matched_none
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let paths = Paths::new([]);

        assert!(paths.matches(""));
        assert!(paths.matches("/"));
        assert!(paths.matches("/foo/bar"));
        assert!(paths.matches("/foo/bar/"));
        assert!(paths.matches("/foo/bar/baz"));
        assert!(paths.matches("/foo/bar/baz/"));
        assert!(paths.matches("/foo"));
        assert!(paths.matches("/foo/"));
        assert!(paths.matches("/foo/bar/baz"));
        assert!(paths.matches("/foo/bar/baz/"));

        let paths = Paths::new([
            Matcher::new(PermissionKind::Acceptable, Kind::exact("/foo/bar")),
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::glob("/foo/bar/*").unwrap(),
            ),
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::glob("/foo/*/baz").unwrap(),
            ),
        ]);

        assert!(paths.matches("/foo/bar"));
        assert!(paths.matches("/foo/bar/"));
        assert!(paths.matches("/foo/bar/baz"));
        assert!(paths.matches("/foo/bar/baz/"));
        assert!(paths.matches("/foo/a/baz"));
        assert!(paths.matches("/foo/a/baz/"));
        assert!(paths.matches("/foo/b/baz"));
        assert!(paths.matches("/foo/b/baz/"));
        assert!(!paths.matches("/foo"));
        assert!(!paths.matches("/foo/"));
        assert!(!paths.matches("/foot/bar/bar"));
        assert!(!paths.matches("/foot/bar/bar/"));
        assert!(!paths.matches("/foo/a/bar"));
        assert!(!paths.matches("/foo/a/bar/"));

        let paths = Paths::new([
            Matcher::new(PermissionKind::Acceptable, Kind::exact("/foo/bar")),
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::glob("/foo/bar/*").unwrap(),
            ),
            Matcher::new(
                PermissionKind::Unacceptable,
                Kind::glob("/foo/*/baz").unwrap(),
            ),
        ]);

        assert!(paths.matches("/foo/bar"));
        assert!(paths.matches("/foo/bar/"));
        assert!(paths.matches("/foo/bar/a"));
        assert!(paths.matches("/foo/bar/a/"));
        assert!(!paths.matches("/foo/bar/baz"));
        assert!(!paths.matches("/foo/bar/baz/"));
        assert!(!paths.matches("/foo/a/baz"));
        assert!(!paths.matches("/foo/a/baz/"));
        assert!(!paths.matches("/foo/b/baz"));
        assert!(!paths.matches("/foo/b/baz/"));
        assert!(!paths.matches("/foo"));
        assert!(!paths.matches("/foo/"));
        assert!(!paths.matches("/foot/bar/bar"));
        assert!(!paths.matches("/foot/bar/bar/"));
        assert!(!paths.matches("/foo/a/bar"));
        assert!(!paths.matches("/foo/a/bar/"));

        let paths = Paths::new([Matcher::new(
            PermissionKind::Acceptable,
            Kind::glob("/*/bar").unwrap(),
        )]);

        assert!(paths.matches("/foo/bar"));
        assert!(paths.matches("/foo/bar/"));
        assert!(paths.matches("/bar/bar"));
        assert!(paths.matches("/bar/bar/"));
        assert!(!paths.matches("/foo"));
        assert!(!paths.matches("/foo/"));
        assert!(!paths.matches("/foo/bar/baz"));
        assert!(!paths.matches("/foo/bar/baz/"));
        assert!(!paths.matches("foo/bar"));
        assert!(!paths.matches("foo/bar/"));
        assert!(!paths.matches("/bar"));
        assert!(!paths.matches("/bar/"));

        let paths = Paths::new([Matcher::new(
            PermissionKind::Acceptable,
            Kind::glob("/*/bar/*").unwrap(),
        )]);

        assert!(paths.matches("/foo/bar/baz"));
        assert!(paths.matches("/foo/bar/baz/"));
        assert!(paths.matches("/bar/bar/baz"));
        assert!(paths.matches("/bar/bar/baz/"));
        assert!(!paths.matches("/foo"));
        assert!(!paths.matches("/foo/"));
        assert!(!paths.matches("/foo/bar"));
        assert!(!paths.matches("/foo/bar/"));
        assert!(!paths.matches("/bar"));
        assert!(!paths.matches("/bar/"));
    }
}