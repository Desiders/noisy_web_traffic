use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};
use url::{ParseError, Url};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootUrl {
    pub value: Url,
}

impl RootUrl {
    pub fn new(value: impl AsRef<str>) -> Result<Self, ParseError> {
        Ok(Self {
            value: Url::parse(value.as_ref())?,
        })
    }
}

impl Display for RootUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl Deref for RootUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<Url> for RootUrl {
    fn as_ref(&self) -> &Url {
        &self.value
    }
}
