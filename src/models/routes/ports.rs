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

    pub fn matches(&self, port: u16) -> bool {
        let matched_any = self.acceptable.iter().any(|kind| kind.matches(port));

        if !matched_any {
            return false;
        }

        let matched_none = self.unacceptable.iter().any(|kind| kind.matches(port));

        !matched_none
    }

    pub fn matches_str(&self, port: impl AsRef<str>) -> bool {
        let port = port.as_ref();

        let matched_any = self.acceptable.iter().any(|kind| kind.matches_str(port));

        if !matched_any {
            return false;
        }

        let matched_none = self.unacceptable.iter().any(|kind| kind.matches_str(port));

        !matched_none
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let ports = Ports::new([]);

        assert!(ports.matches(80));
        assert!(ports.matches_str("80"));
        assert!(ports.matches(8080));
        assert!(ports.matches_str("8080"));

        let ports = Ports::new(vec![
            Matcher::new(PermissionKind::Acceptable, Kind::exact(80)),
            Matcher::new(PermissionKind::Acceptable, Kind::glob("8?8?").unwrap()),
            Matcher::new(PermissionKind::Unacceptable, Kind::exact(8080)),
        ]);

        assert!(ports.matches(80));
        assert!(ports.matches_str("80"));
        assert!(ports.matches(8081));
        assert!(ports.matches_str("8081"));
        assert!(ports.matches(8180));
        assert!(ports.matches_str("8180"));
        assert!(!ports.matches(8071));
        assert!(!ports.matches_str("8071"));
        assert!(!ports.matches(8080));
        assert!(!ports.matches_str("8080"));

        let ports = Ports::new(vec![
            Matcher::new(PermissionKind::Acceptable, Kind::exact(80)),
            Matcher::new(PermissionKind::Acceptable, Kind::glob("8?8?").unwrap()),
            Matcher::new(PermissionKind::Unacceptable, Kind::exact(8080)),
            Matcher::new(PermissionKind::Unacceptable, Kind::exact(80)),
        ]);

        assert!(ports.matches(8081));
        assert!(ports.matches_str("8081"));
        assert!(ports.matches(8180));
        assert!(ports.matches_str("8180"));
        assert!(!ports.matches(80));
        assert!(!ports.matches_str("80"));
        assert!(!ports.matches(8071));
        assert!(!ports.matches_str("8071"));
        assert!(!ports.matches(8080));
        assert!(!ports.matches_str("8080"));
    }
}
