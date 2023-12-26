pub mod depth;
pub mod proxies;
pub mod proxy;
pub mod redirections;
pub mod time;
pub mod user_agent;
pub mod user_agents;

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

#[derive(Debug, Clone)]
pub struct Polling {
    pub depth: depth::Depth,
    pub proxies: proxies::Proxies,
    pub redirections: redirections::Redirections,
    pub time: time::Time,
    pub user_agents: user_agents::UserAgents,
}

impl Polling {
    pub fn new(
        depth: depth::Depth,
        proxies: proxies::Proxies,
        redirections: redirections::Redirections,
        time: time::Time,
        user_agents: user_agents::UserAgents,
    ) -> Self {
        Self {
            depth,
            proxies,
            redirections,
            time,
            user_agents,
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
            "Polling {{ depth: {}, proxies: {}, redirections: {}, time: {}, user_agents: {} }}",
            self.depth, self.proxies, self.redirections, self.time, self.user_agents
        )
    }
}

impl Default for Polling {
    fn default() -> Self {
        Self::new(
            depth::Depth::default(),
            proxies::Proxies::default(),
            redirections::Redirections::default(),
            time::Time::default(),
            user_agents::UserAgents::default(),
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    depth: depth::Depth,
    proxies: proxies::Proxies,
    redirections: redirections::Redirections,
    time: time::Time,
    user_agents: user_agents::UserAgents,
}

impl Builder {
    pub fn depth(mut self, depth: depth::Depth) -> Self {
        self.depth = depth;
        self
    }

    pub fn proxy(mut self, proxy: proxy::Proxy) -> Self {
        self.proxies.extend(iter::once(proxy));
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

    pub fn user_agent(mut self, user_agent: user_agent::UserAgent) -> Self {
        self.user_agents.extend(iter::once(user_agent));
        self
    }

    pub fn build(self) -> Polling {
        Polling::new(
            self.depth,
            self.proxies,
            self.redirections,
            self.time,
            self.user_agents,
        )
    }
}
