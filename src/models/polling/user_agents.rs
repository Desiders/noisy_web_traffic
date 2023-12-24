use super::user_agent::UserAgent;

use rand::{seq::SliceRandom as _, thread_rng};
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UserAgents(pub Vec<UserAgent>);

impl UserAgents {
    pub fn new(user_agents: impl IntoIterator<Item = UserAgent>) -> Self {
        Self(user_agents.into_iter().collect())
    }

    pub fn get_random(&self) -> Option<&UserAgent> {
        let mut rng = thread_rng();

        self.0.choose(&mut rng)
    }

    pub fn extend(&mut self, user_agents: impl IntoIterator<Item = UserAgent>) {
        self.0.extend(user_agents);
    }
}

impl Display for UserAgents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let user_agents = self.0.iter().map(ToString::to_string).collect::<Vec<_>>();

        write!(f, "{}", user_agents.join(", "))
    }
}

impl Deref for UserAgents {
    type Target = Vec<UserAgent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
