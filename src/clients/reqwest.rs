use reqwest::{Body, Response};

#[derive(Debug, Clone)]
pub struct Reqwest {
    client: reqwest::Client,
}

impl Reqwest {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        let response = self.client.get(url.as_ref()).send().await?;

        Ok(response)
    }

    pub async fn post(
        &self,
        url: impl AsRef<str>,
        body: impl Into<Body>,
    ) -> Result<Response, reqwest::Error> {
        let response = self.client.post(url.as_ref()).body(body).send().await?;

        Ok(response)
    }

    pub async fn put(
        &self,
        url: impl AsRef<str>,
        body: impl Into<Body>,
    ) -> Result<Response, reqwest::Error> {
        let response = self.client.put(url.as_ref()).body(body).send().await?;

        Ok(response)
    }

    pub async fn delete(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        let response = self.client.delete(url.as_ref()).send().await?;

        Ok(response)
    }

    pub async fn patch(
        &self,
        url: impl AsRef<str>,
        body: impl Into<Body>,
    ) -> Result<Response, reqwest::Error> {
        let response = self.client.patch(url.as_ref()).body(body).send().await?;

        Ok(response)
    }

    pub async fn head(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        let response = self.client.head(url.as_ref()).send().await?;

        Ok(response)
    }

    pub async fn options(&self, url: impl AsRef<str>) -> Result<Response, reqwest::Error> {
        let response = self
            .client
            .request(reqwest::Method::OPTIONS, url.as_ref())
            .send()
            .await?;

        Ok(response)
    }
}

impl Default for Reqwest {
    fn default() -> Self {
        Self::new()
    }
}
