use crate::models::polling::{
    proxy::Proxy, redirections::Redirections, time::Time, user_agent::UserAgent,
};

use reqwest::{self, redirect::Policy, Client, Response};
use std::time::Duration;
use tracing::instrument;

pub struct Reqwest {
    user_agent: Option<UserAgent>,
    client: Client,
}

impl Reqwest {
    pub fn new(
        proxy: Option<Proxy>,
        max_redirects: usize,
        request_timeout: u64,
        user_agent: Option<UserAgent>,
    ) -> Result<Self, reqwest::Error> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_millis(request_timeout))
            .redirect(if max_redirects > 0 {
                Policy::limited(max_redirects)
            } else {
                Policy::none()
            });

        if let Some(proxy) = proxy {
            client_builder = client_builder.proxy(reqwest::Proxy::all(proxy.as_ref())?);
        }

        if let Some(ref user_agent) = user_agent {
            client_builder = client_builder.user_agent(user_agent.as_ref());
        }

        Ok(Self {
            user_agent,
            client: client_builder.build()?,
        })
    }

    pub const fn user_agent(&self) -> Option<&UserAgent> {
        self.user_agent.as_ref()
    }

    #[instrument(skip_all, fields(url = %url.as_ref()))]
    pub async fn get(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        self.client.get(url.as_ref()).send().await
    }
}

impl Default for Reqwest {
    fn default() -> Self {
        Self::new(
            None,
            Redirections::default().max_redirects().into(),
            Time::default().request_timeout,
            None,
        )
        .expect("Failed to create default Reqwest client")
    }
}
