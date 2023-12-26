use super::proxy::Proxy;

use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Proxies(pub Vec<Proxy>);

impl Proxies {
    pub fn extend(&mut self, proxies: impl IntoIterator<Item = Proxy>) {
        self.0.extend(proxies);
    }
}

impl Display for Proxies {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let proxies = self.0.iter().map(ToString::to_string).collect::<Vec<_>>();

        write!(f, "{}", proxies.join(", "))
    }
}

impl Deref for Proxies {
    type Target = Vec<Proxy>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
