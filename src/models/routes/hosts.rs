use super::{
    host::{Kind, Matcher},
    permission::Kind as PermissionKind,
};

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Hosts {
    pub acceptable: Vec<Kind>,
    pub unacceptable: Vec<Kind>,
}

impl Hosts {
    #[allow(dead_code)]
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

    pub fn matches(&self, host: impl AsRef<str>) -> bool {
        let host = host.as_ref();

        let matched_any = self.acceptable.iter().any(|kind| kind.matches(host));

        if !matched_any {
            return false;
        }

        let matched_none = self.unacceptable.iter().any(|kind| kind.matches(host));

        !matched_none
    }
}

impl Display for Hosts {
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
            "Hosts {{ acceptable: [{}], unacceptable: [{}] }}",
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
        let hosts = Hosts::new([]);

        assert!(hosts.matches("example.com"));
        assert!(hosts.matches("www.example.com"));
        assert!(hosts.matches("api.example.com"));
        assert!(hosts.matches("www.example.org"));
        assert!(hosts.matches("api.example.org"));
        assert!(hosts.matches("example"));
        assert!(hosts.matches("example.org"));

        let hosts = Hosts::new([
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::exact("example.com").unwrap(),
            ),
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::glob("*.example.com").unwrap(),
            ),
        ]);

        assert!(hosts.matches("example.com"));
        assert!(hosts.matches("www.example.com"));
        assert!(hosts.matches("api.example.com"));
        assert!(hosts.matches(".example.com"));
        assert!(!hosts.matches("www.example.org"));
        assert!(!hosts.matches("api.example.org"));
        assert!(!hosts.matches("example"));
        assert!(!hosts.matches("example.org"));

        let hosts = Hosts::new([
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::exact("example.com").unwrap(),
            ),
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::glob("*.example.com").unwrap(),
            ),
            Matcher::new(
                PermissionKind::Unacceptable,
                Kind::exact("api.example.com").unwrap(),
            ),
        ]);

        assert!(hosts.matches("example.com"));
        assert!(hosts.matches("www.example.com"));
        assert!(!hosts.matches("api.example.com"));
        assert!(!hosts.matches("www.example.org"));
        assert!(!hosts.matches("api.example.org"));
        assert!(!hosts.matches("example"));
        assert!(!hosts.matches("example.org"));

        let hosts = Hosts::new([
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::exact("example.com").unwrap(),
            ),
            Matcher::new(
                PermissionKind::Unacceptable,
                Kind::glob("*.example.com").unwrap(),
            ),
        ]);

        assert!(hosts.matches("example.com"));
        assert!(!hosts.matches(".example.com"));
        assert!(!hosts.matches("www.example.com"));
        assert!(!hosts.matches("api.example.com"));
        assert!(!hosts.matches("www.example.org"));
        assert!(!hosts.matches("api.example.org"));
        assert!(!hosts.matches("example"));
        assert!(!hosts.matches("example.org"));

        let hosts = Hosts::new([
            Matcher::new(
                PermissionKind::Acceptable,
                Kind::exact("example.com").unwrap(),
            ),
            Matcher::new(
                PermissionKind::Unacceptable,
                Kind::glob("example.*").unwrap(),
            ),
        ]);

        assert!(!hosts.matches("example.com"));
        assert!(!hosts.matches("example.com."));
        assert!(!hosts.matches("www.example.com"));
        assert!(!hosts.matches("api.example.com"));
        assert!(!hosts.matches("www.example.org"));
        assert!(!hosts.matches("api.example.org"));
        assert!(!hosts.matches("example"));
        assert!(!hosts.matches("example.org"));
    }
}
