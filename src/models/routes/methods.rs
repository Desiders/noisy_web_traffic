use super::{
    method::{Kind, Matcher},
    permission::Kind as PermissionKind,
};

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Methods {
    pub acceptable: Vec<Kind>,
    pub unacceptable: Vec<Kind>,
}

impl Methods {
    pub fn new(methods: impl IntoIterator<Item = Matcher>) -> Self {
        let mut acceptable = vec![];
        let mut unacceptable = vec![];

        for method in methods {
            match method.permission {
                PermissionKind::Acceptable => acceptable.push(method.kind),
                PermissionKind::Unacceptable => unacceptable.push(method.kind),
            }
        }

        if acceptable.is_empty() {
            acceptable.push(Kind::AnySupported);
        }

        Self {
            acceptable,
            unacceptable,
        }
    }

    pub fn extend(&mut self, methods: impl IntoIterator<Item = Matcher>) {
        for method in methods {
            match method.permission {
                PermissionKind::Acceptable => self.acceptable.push(method.kind),
                PermissionKind::Unacceptable => self.unacceptable.push(method.kind),
            }
        }
    }

    pub fn matches(&self, method: impl AsRef<str>) -> bool {
        let method = method.as_ref();

        let matched_any = self.acceptable.iter().any(|kind| kind.matches(method));

        if !matched_any {
            return false;
        }

        let matched_none = self.unacceptable.iter().any(|kind| kind.matches(method));

        !matched_none
    }
}

impl Display for Methods {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut acceptable = self
            .acceptable
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        acceptable.sort();

        let mut unacceptable = self
            .unacceptable
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        unacceptable.sort();

        write!(
            f,
            "Methods {{ acceptable: [{}], unacceptable: [{}] }}",
            acceptable.join(", "),
            unacceptable.join(", "),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let methods = Methods::new([]);

        assert!(methods.matches("get"));
        assert!(methods.matches("GET"));
        assert!(methods.matches("post"));
        assert!(methods.matches("POST"));
        assert!(methods.matches("put"));
        assert!(methods.matches("PUT"));
        assert!(methods.matches("patch"));
        assert!(methods.matches("PATCH"));
        assert!(methods.matches("delete"));
        assert!(methods.matches("DELETE"));
        assert!(methods.matches("head"));
        assert!(methods.matches("HEAD"));
        assert!(methods.matches("options"));
        assert!(methods.matches("OPTIONS"));
        assert!(!methods.matches("foo"));
        assert!(!methods.matches("bar"));
        assert!(!methods.matches("baz"));

        let methods = Methods::new([
            Matcher::new(PermissionKind::Acceptable, Kind::Get),
            Matcher::new(PermissionKind::Acceptable, Kind::Post),
            Matcher::new(PermissionKind::Acceptable, Kind::Put),
            Matcher::new(PermissionKind::Acceptable, Kind::Patch),
            Matcher::new(PermissionKind::Acceptable, Kind::Delete),
        ]);

        assert!(methods.matches("get"));
        assert!(methods.matches("GET"));
        assert!(methods.matches("post"));
        assert!(methods.matches("POST"));
        assert!(methods.matches("put"));
        assert!(methods.matches("PUT"));
        assert!(methods.matches("patch"));
        assert!(methods.matches("PATCH"));
        assert!(methods.matches("delete"));
        assert!(methods.matches("DELETE"));
        assert!(!methods.matches("head"));
        assert!(!methods.matches("HEAD"));
        assert!(!methods.matches("options"));
        assert!(!methods.matches("OPTIONS"));
        assert!(!methods.matches("foo"));
        assert!(!methods.matches("bar"));
        assert!(!methods.matches("baz"));

        let methods = Methods::new([Matcher::new(PermissionKind::Acceptable, Kind::AnySupported)]);

        assert!(methods.matches("get"));
        assert!(methods.matches("GET"));
        assert!(methods.matches("post"));
        assert!(methods.matches("POST"));
        assert!(methods.matches("put"));
        assert!(methods.matches("PUT"));
        assert!(methods.matches("patch"));
        assert!(methods.matches("PATCH"));
        assert!(methods.matches("delete"));
        assert!(methods.matches("DELETE"));
        assert!(methods.matches("head"));
        assert!(methods.matches("HEAD"));
        assert!(methods.matches("options"));
        assert!(methods.matches("OPTIONS"));
        assert!(!methods.matches("foo"));
        assert!(!methods.matches("bar"));
        assert!(!methods.matches("baz"));

        let methods = Methods::new([
            Matcher::new(PermissionKind::Acceptable, Kind::AnySupported),
            Matcher::new(PermissionKind::Unacceptable, Kind::Head),
        ]);

        assert!(methods.matches("get"));
        assert!(methods.matches("GET"));
        assert!(methods.matches("post"));
        assert!(methods.matches("POST"));
        assert!(methods.matches("put"));
        assert!(methods.matches("PUT"));
        assert!(methods.matches("patch"));
        assert!(methods.matches("PATCH"));
        assert!(methods.matches("delete"));
        assert!(methods.matches("DELETE"));
        assert!(methods.matches("options"));
        assert!(methods.matches("OPTIONS"));
        assert!(!methods.matches("head"));
        assert!(!methods.matches("HEAD"));
        assert!(!methods.matches("foo"));
        assert!(!methods.matches("bar"));
        assert!(!methods.matches("baz"));
    }
}
