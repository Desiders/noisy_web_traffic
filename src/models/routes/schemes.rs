use super::{
    permission::Kind as PermissionKind,
    scheme::{Kind, Matcher},
};

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Schemes {
    pub acceptable: Vec<Kind>,
    pub unacceptable: Vec<Kind>,
}

impl Schemes {
    #[allow(dead_code)]
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

    pub fn matches(&self, scheme: impl AsRef<str>) -> bool {
        let scheme = scheme.as_ref();

        let matched_any = self.acceptable.iter().any(|kind| kind.matches(scheme));

        if !matched_any {
            return false;
        }

        let matched_none = self.unacceptable.iter().any(|kind| kind.matches(scheme));

        !matched_none
    }
}

impl Display for Schemes {
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
            "Schemes {{ acceptable: [{}], unacceptable: [{}] }}",
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
        let schemes = Schemes::new([]);

        assert!(schemes.matches("http"));
        assert!(schemes.matches("HTTP"));
        assert!(schemes.matches("HtTP"));
        assert!(schemes.matches("https"));
        assert!(schemes.matches("HTTPS"));
        assert!(schemes.matches("HtTPS"));
        assert!(!schemes.matches("ftp"));
        assert!(!schemes.matches("FTP"));
        assert!(!schemes.matches("FtP"));
        assert!(!schemes.matches("qwe"));
        assert!(!schemes.matches("QWE"));
        assert!(!schemes.matches("QwE"));

        let schemes = Schemes::new([
            Matcher::new(PermissionKind::Acceptable, Kind::Http),
            Matcher::new(PermissionKind::Acceptable, Kind::Https),
        ]);

        assert!(schemes.matches("http"));
        assert!(schemes.matches("HTTP"));
        assert!(schemes.matches("HtTP"));
        assert!(schemes.matches("https"));
        assert!(schemes.matches("HTTPS"));
        assert!(schemes.matches("HtTPS"));
        assert!(!schemes.matches("ftp"));
        assert!(!schemes.matches("FTP"));
        assert!(!schemes.matches("FtP"));
        assert!(!schemes.matches("qwe"));
        assert!(!schemes.matches("QWE"));
        assert!(!schemes.matches("QwE"));

        let schemes = Schemes::new([
            Matcher::new(PermissionKind::Acceptable, Kind::Http),
            Matcher::new(PermissionKind::Unacceptable, Kind::Https),
        ]);

        assert!(schemes.matches("http"));
        assert!(schemes.matches("HTTP"));
        assert!(schemes.matches("HtTP"));
        assert!(!schemes.matches("https"));
        assert!(!schemes.matches("HTTPS"));
        assert!(!schemes.matches("HtTPS"));
        assert!(!schemes.matches("ftp"));
        assert!(!schemes.matches("FTP"));
        assert!(!schemes.matches("FtP"));
        assert!(!schemes.matches("qwe"));
        assert!(!schemes.matches("QWE"));
        assert!(!schemes.matches("QwE"));
    }
}
