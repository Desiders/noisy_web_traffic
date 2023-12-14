use super::{
    method::{Kind, Matcher},
    permission::Kind as PermissionKind,
};

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
