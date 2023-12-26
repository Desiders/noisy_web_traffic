use crate::models::polling::{
    proxies::Proxies, redirections::Redirections, time::Time, user_agents::UserAgents,
};

use rand::{seq::SliceRandom as _, thread_rng};
use reqwest::{self, redirect::Policy, Body, Client, ClientBuilder, Proxy, Response};
use std::time::Duration;
use tracing::instrument;

pub struct Reqwest {
    proxies: Vec<Proxy>,
    user_agents: UserAgents,
    // We use a Box here because ClientBuilder doesn't implement Clone and we need to customize the client for each request
    client_builder_fn: Box<dyn Fn() -> ClientBuilder + Send + Sync>,
}

impl Reqwest {
    pub fn new(
        proxies: &Proxies,
        max_redirects: usize,
        request_timeout: u64,
        user_agents: UserAgents,
    ) -> Result<Self, reqwest::Error> {
        let proxies = proxies
            .iter()
            .map(|proxy| Proxy::all(proxy.as_ref()))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            proxies,
            user_agents,
            client_builder_fn: Box::new(move || {
                let redirect_policy = if max_redirects > 0 {
                    Policy::limited(max_redirects)
                } else {
                    Policy::none()
                };

                Client::builder()
                    .timeout(Duration::from_millis(request_timeout))
                    .redirect(redirect_policy)
            }),
        })
    }

    pub fn default() -> Result<Self, reqwest::Error> {
        Self::new(
            &Proxies::default(),
            Redirections::default().max_redirects().into(),
            Time::default().request_timeout,
            UserAgents::default(),
        )
    }

    fn get_random_proxy(&self) -> Option<&Proxy> {
        self.proxies.choose(&mut thread_rng())
    }

    fn get_random_user_agent(&self) -> Option<&str> {
        Some(self.user_agents.get_random()?.as_ref())
    }

    fn get_client(&self) -> Result<Client, reqwest::Error> {
        let mut client_builder = (self.client_builder_fn)();

        if let Some(proxy) = self.get_random_proxy() {
            client_builder = client_builder.proxy(proxy.clone());
        }

        if let Some(user_agent) = self.get_random_user_agent() {
            client_builder = client_builder.user_agent(user_agent);
        }

        client_builder.build()
    }

    #[instrument(skip_all, fields(url = %url.as_ref()))]
    pub async fn get(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        let response = self.get_client()?.get(url.as_ref()).send().await?;

        Ok(response)
    }

    #[instrument(skip_all, fields(url = %url.as_ref()))]
    pub async fn post(
        &self,
        url: impl AsRef<str>,
        body: impl Into<Body>,
    ) -> Result<Response, reqwest::Error> {
        let response = self
            .get_client()?
            .post(url.as_ref())
            .body(body)
            .send()
            .await?;

        Ok(response)
    }

    #[instrument(skip_all, fields(url = %url.as_ref()))]
    pub async fn put(
        &self,
        url: impl AsRef<str>,
        body: impl Into<Body>,
    ) -> Result<Response, reqwest::Error> {
        let response = self
            .get_client()?
            .put(url.as_ref())
            .body(body)
            .send()
            .await?;

        Ok(response)
    }

    #[instrument(skip_all, fields(url = %url.as_ref()))]
    pub async fn delete(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        let response = self.get_client()?.delete(url.as_ref()).send().await?;

        Ok(response)
    }

    #[instrument(skip_all, fields(url = %url.as_ref()))]
    pub async fn patch(
        &self,
        url: impl AsRef<str>,
        body: impl Into<Body>,
    ) -> Result<Response, reqwest::Error> {
        let response = self
            .get_client()?
            .patch(url.as_ref())
            .body(body)
            .send()
            .await?;

        Ok(response)
    }

    #[instrument(skip_all, fields(url = %url.as_ref()))]
    pub async fn head(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        let response = self.get_client()?.head(url.as_ref()).send().await?;

        Ok(response)
    }

    #[instrument(skip_all, fields(url = %url.as_ref()))]
    pub async fn options(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        let response = self
            .get_client()?
            .request(reqwest::Method::OPTIONS, url.as_ref())
            .send()
            .await?;

        Ok(response)
    }
}
