use super::permission::Kind as PermissionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Http,
    Https,
    AnySupported, // This is a special case that matches all schemes above
}

impl Kind {
    pub fn matches(&self, scheme: impl AsRef<str>) -> bool {
        let scheme = scheme.as_ref().to_lowercase();

        match self {
            Kind::Http => scheme == "http",
            Kind::Https => scheme == "https",
            Kind::AnySupported => scheme == "http" || scheme == "https",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Matcher {
    pub permission: PermissionKind,
    pub kind: Kind,
}

impl Matcher {
    pub const fn new(permission: PermissionKind, kind: Kind) -> Self {
        Self { permission, kind }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unsupported scheme: {0}")]
pub struct UnsupportedSchemeError(String);

impl TryFrom<String> for Kind {
    type Error = UnsupportedSchemeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();

        match value.as_str() {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            _ => Err(UnsupportedSchemeError(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let scheme = Kind::Http;

        assert!(scheme.matches("http"));
        assert!(scheme.matches("HTTP"));
        assert!(scheme.matches("HtTP"));
        assert!(!scheme.matches("https"));
        assert!(!scheme.matches("HTTPS"));
        assert!(!scheme.matches("HtTPs"));

        let scheme = Kind::Https;

        assert!(scheme.matches("https"));
        assert!(scheme.matches("HTTPS"));
        assert!(scheme.matches("HtTPs"));
        assert!(!scheme.matches("http"));
        assert!(!scheme.matches("HTTP"));
        assert!(!scheme.matches("HtTP"));

        let scheme = Kind::AnySupported;

        assert!(scheme.matches("http"));
        assert!(scheme.matches("HTTP"));
        assert!(scheme.matches("HtTP"));
        assert!(scheme.matches("https"));
        assert!(scheme.matches("HTTPS"));
        assert!(scheme.matches("HtTPs"));
        assert!(!scheme.matches("ftp"));
        assert!(!scheme.matches("FTP"));
        assert!(!scheme.matches("FtP"));
        assert!(!scheme.matches("qwe"));
        assert!(!scheme.matches("QWE"));
        assert!(!scheme.matches("QwE"));
    }
}
