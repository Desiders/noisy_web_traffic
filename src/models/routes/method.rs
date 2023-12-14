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
    pub fn matches(&self, method: impl AsRef<str>) -> bool {
        let method = method.as_ref().to_lowercase();

        match self {
            Kind::Get => method == "get",
            Kind::Post => method == "post",
            Kind::Put => method == "put",
            Kind::Patch => method == "patch",
            Kind::Delete => method == "delete",
            Kind::Head => method == "head",
            Kind::Options => method == "options",
            Kind::AnySupported => true,
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
