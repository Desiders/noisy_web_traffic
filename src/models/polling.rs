pub mod depth;
pub mod proxy;
pub mod redirections;
pub mod time;
pub mod user_agent;

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Polling {
    pub depth: depth::Depth,
    pub proxy: Option<proxy::Proxy>,
    pub redirections: redirections::Redirections,
    pub time: time::Time,
    pub user_agent: Option<user_agent::UserAgent>,
}

impl Polling {
    pub fn new(
        depth: depth::Depth,
        proxy: Option<proxy::Proxy>,
        redirections: redirections::Redirections,
        time: time::Time,
        user_agent: Option<user_agent::UserAgent>,
    ) -> Self {
        Self {
            depth,
            proxy,
            redirections,
            time,
            user_agent,
        }
    }

    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn depth_matches(&self, depth: u16) -> bool {
        self.depth.matches(depth)
    }
}

impl Display for Polling {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Polling {{ depth: {}, proxy: {}, redirections: {}, time: {}, user_agent: {} }}",
            self.depth,
            self.proxy.as_ref().map(|p| p.as_ref()).unwrap_or("None"),
            self.redirections,
            self.time,
            self.user_agent
                .as_ref()
                .map(|ua| ua.as_ref())
                .unwrap_or("None")
        )
    }
}

impl Default for Polling {
    fn default() -> Self {
        Self::new(
            depth::Depth::default(),
            None,
            redirections::Redirections::default(),
            time::Time::default(),
            None,
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    depth: depth::Depth,
    proxy: Option<proxy::Proxy>,
    redirections: redirections::Redirections,
    time: time::Time,
    user_agent: Option<user_agent::UserAgent>,
}

impl Builder {
    pub fn depth(mut self, depth: depth::Depth) -> Self {
        self.depth = depth;
        self
    }

    pub fn proxy(mut self, proxy: Option<proxy::Proxy>) -> Self {
        self.proxy = proxy;
        self
    }

    pub fn redirections(mut self, redirections: redirections::Redirections) -> Self {
        self.redirections = redirections;
        self
    }

    pub fn time(mut self, time: time::Time) -> Self {
        self.time = time;
        self
    }

    pub fn user_agent(mut self, user_agent: Option<user_agent::UserAgent>) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn build(self) -> Polling {
        Polling::new(
            self.depth,
            self.proxy,
            self.redirections,
            self.time,
            self.user_agent,
        )
    }
}
