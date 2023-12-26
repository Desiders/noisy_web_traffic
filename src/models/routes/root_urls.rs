use super::root_url::RootUrl;

use rand::{seq::SliceRandom as _, thread_rng};
use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::Deref,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RootUrls(pub Vec<RootUrl>);

impl RootUrls {
    pub fn get_random(&self) -> Option<&RootUrl> {
        let mut rng = thread_rng();

        self.0.choose(&mut rng)
    }

    pub fn extend(&mut self, root_urls: impl IntoIterator<Item = RootUrl>) {
        self.0.extend(root_urls);
    }
}

impl Display for RootUrls {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut root_urls = self.0.iter();

        if let Some(root_url) = root_urls.next() {
            write!(f, "{root_url}")?;

            for root_url in root_urls {
                write!(f, ", {root_url}")?;
            }
        }

        Ok(())
    }
}

impl Deref for RootUrls {
    type Target = Vec<RootUrl>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[RootUrl]> for RootUrls {
    fn as_ref(&self) -> &[RootUrl] {
        &self.0
    }
}
