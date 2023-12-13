use super::{
    permission::Kind as PermissionKind,
    port::{Kind, Matcher},
};

#[derive(Debug, Default, Clone)]
pub struct Ports {
    pub acceptable: Vec<Kind>,
    pub unacceptable: Vec<Kind>,
}

impl Ports {
    pub fn new(ports: impl IntoIterator<Item = Matcher>) -> Self {
        let mut acceptable = vec![];
        let mut unacceptable = vec![];

        for port in ports {
            match port.permission {
                PermissionKind::Acceptable => acceptable.push(port.kind),
                PermissionKind::Unacceptable => unacceptable.push(port.kind),
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

    pub fn extend(&mut self, ports: impl IntoIterator<Item = Matcher>) {
        for port in ports {
            match port.permission {
                PermissionKind::Acceptable => self.acceptable.push(port.kind),
                PermissionKind::Unacceptable => self.unacceptable.push(port.kind),
            }
        }
    }
}
