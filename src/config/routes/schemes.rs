use super::{
    permission::Kind as PermissionKind,
    scheme::{Kind, Matcher},
};

#[derive(Debug, Default, Clone)]
pub struct Schemes {
    pub acceptable: Vec<Kind>,
    pub unacceptable: Vec<Kind>,
}

impl Schemes {
    pub fn new(schemes: impl IntoIterator<Item = Matcher>) -> Self {
        let mut acceptable = vec![];
        let mut unacceptable = vec![];

        for scheme in schemes {
            match scheme.permission {
                PermissionKind::Acceptable => acceptable.push(scheme.kind),
                PermissionKind::Unacceptable => unacceptable.push(scheme.kind),
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

    pub fn extend(&mut self, schemes: impl IntoIterator<Item = Matcher>) {
        for scheme in schemes {
            match scheme.permission {
                PermissionKind::Acceptable => self.acceptable.push(scheme.kind),
                PermissionKind::Unacceptable => self.unacceptable.push(scheme.kind),
            }
        }
    }
}
