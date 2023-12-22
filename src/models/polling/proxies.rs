use super::proxy::Proxy;

use rand::{seq::SliceRandom as _, thread_rng};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Proxies(pub Vec<Proxy>);

impl Proxies {
    pub fn new(proxies: impl IntoIterator<Item = Proxy>) -> Self {
        Self(proxies.into_iter().collect())
    }

    pub fn get_random(&self) -> Option<&Proxy> {
        let mut rng = thread_rng();

        self.0.choose(&mut rng)
    }

    pub fn extend(&mut self, proxies: impl IntoIterator<Item = Proxy>) {
        self.0.extend(proxies);
    }
}
