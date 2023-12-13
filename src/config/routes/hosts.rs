use super::{
    host::{Kind, Matcher},
    permission::Kind as PermissionKind,
};

#[derive(Debug, Default, Clone)]
pub struct Hosts {
    pub acceptable: Vec<Kind>,
    pub unacceptable: Vec<Kind>,
}

impl Hosts {
    pub fn new(hosts: impl IntoIterator<Item = Matcher>) -> Self {
        let mut acceptable = vec![];
        let mut unacceptable = vec![];

        for host in hosts {
            match host.permission {
                PermissionKind::Acceptable => acceptable.push(host.kind),
                PermissionKind::Unacceptable => unacceptable.push(host.kind),
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

    pub fn extend(&mut self, hosts: impl IntoIterator<Item = Matcher>) {
        for host in hosts {
            match host.permission {
                PermissionKind::Acceptable => self.acceptable.push(host.kind),
                PermissionKind::Unacceptable => self.unacceptable.push(host.kind),
            }
        }
    }
}
