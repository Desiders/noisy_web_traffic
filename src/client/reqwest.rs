use reqwest::Body;
use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

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

    pub async fn get<T>(&self, url: impl AsRef<str>) -> Result<T, ErrorKind>
    where
        T: DeserializeOwned,
    {
        let response = self.client.get(url.as_ref()).send().await?.json().await?;

        Ok(response)
    }

    pub async fn post<T>(&self, url: impl AsRef<str>, body: impl Into<Body>) -> Result<T, ErrorKind>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .post(url.as_ref())
            .body(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn put<T>(&self, url: impl AsRef<str>, body: impl Into<Body>) -> Result<T, ErrorKind>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .put(url.as_ref())
            .body(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn delete<T>(&self, url: impl AsRef<str>) -> Result<T, ErrorKind>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .delete(url.as_ref())
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn patch<T>(
        &self,
        url: impl AsRef<str>,
        body: impl Into<Body>,
    ) -> Result<T, ErrorKind>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .patch(url.as_ref())
            .body(body)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn head<T>(&self, url: impl AsRef<str>) -> Result<T, ErrorKind>
    where
        T: DeserializeOwned,
    {
        let response = self.client.head(url.as_ref()).send().await?.json().await?;

        Ok(response)
    }

    pub async fn options<T>(&self, url: impl AsRef<str>) -> Result<T, ErrorKind>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .request(reqwest::Method::OPTIONS, url.as_ref())
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
}

impl Default for Reqwest {
    fn default() -> Self {
        Self::new()
    }
}
