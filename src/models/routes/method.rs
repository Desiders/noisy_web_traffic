use super::permission::Kind as PermissionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    AnySupported, // This is a special case that matches all methods above
}

impl Kind {
    pub fn matches(self, method: impl AsRef<str>) -> bool {
        let method = method.as_ref().to_lowercase();

        match self {
            Kind::Get => method == "get",
            Kind::Post => method == "post",
            Kind::Put => method == "put",
            Kind::Patch => method == "patch",
            Kind::Delete => method == "delete",
            Kind::Head => method == "head",
            Kind::Options => method == "options",
            Kind::AnySupported => {
                method == "get"
                    || method == "post"
                    || method == "put"
                    || method == "patch"
                    || method == "delete"
                    || method == "head"
                    || method == "options"
            }
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
#[error("Unsupported method: {0}")]
pub struct UnsupportedMethodError(String);

impl TryFrom<String> for Kind {
    type Error = UnsupportedMethodError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();

        match value.as_str() {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "patch" => Ok(Self::Patch),
            "delete" => Ok(Self::Delete),
            "head" => Ok(Self::Head),
            "options" => Ok(Self::Options),
            _ => Err(UnsupportedMethodError(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let method = Kind::Get;

        assert!(method.matches("get"));
        assert!(method.matches("GET"));
        assert!(method.matches("GeT"));
        assert!(!method.matches("post"));

        let method = Kind::Post;

        assert!(method.matches("post"));
        assert!(method.matches("POST"));
        assert!(method.matches("PoSt"));
        assert!(!method.matches("get"));

        let method = Kind::AnySupported;

        assert!(method.matches("get"));
        assert!(method.matches("GET"));
        assert!(method.matches("GeT"));
        assert!(method.matches("post"));
        assert!(method.matches("POST"));
        assert!(method.matches("PoSt"));
        assert!(method.matches("put"));
        assert!(method.matches("patch"));
        assert!(method.matches("delete"));
        assert!(method.matches("head"));
        assert!(method.matches("options"));
    }
}
