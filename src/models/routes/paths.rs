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
