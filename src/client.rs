use log::{debug, info};
use reqwest::{
    blocking::{Client as ReqwClient, RequestBuilder, Response},
    header::USER_AGENT,
    redirect::Policy,
    Error as ReqwError,
};
use std::time::{Duration, Instant};

pub struct Client {
    reqw: ReqwClient,
    user_agent: Option<String>,
    generate_user_agent: bool,
}

impl Client {
    #[must_use]
    pub fn new(
        max_timeout: u32,
        max_redirections: u32,
        user_agent: &Option<String>,
        generate_user_agent: bool,
    ) -> Self {
        Client {
            reqw: ReqwClient::builder()
                .redirect(Policy::limited(max_redirections as usize))
                .timeout(Duration::from_secs(u64::from(max_timeout)))
                .build()
                .unwrap(),
            user_agent: user_agent.clone(),
            generate_user_agent,
        }
    }

    #[must_use]
    fn generate_user_agent(&self) -> String {
        todo!("Generate user agent");
    }

    #[must_use]
    fn get_user_agent(&self) -> Option<String> {
        if self.generate_user_agent {
            Some(self.generate_user_agent())
        } else {
            self.user_agent.as_ref().map(ToString::to_string)
        }
    }

    fn send(&self, mut builder: RequestBuilder) -> Result<Response, ReqwError> {
        if let Some(user_agent) = self.get_user_agent() {
            builder = builder.header(USER_AGENT, user_agent);
        }
        builder.send()
    }

    pub fn get(&self, url: &str) -> Result<Response, ReqwError> {
        info!("Sending request to `{}`", url);

        let now = Instant::now();
        let response = self.send(self.reqw.get(url));
        debug!("Crawling url took {} seconds", now.elapsed().as_secs_f32());

        response
    }
}
