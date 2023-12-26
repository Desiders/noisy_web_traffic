use crate::{
    clients::reqwest::Reqwest,
    crawlers::urls::{Crawler, ErrorKind as CrawlErrorKind},
    models::{polling::Polling as PollingRules, route::Route, routes::root_urls::RootUrls},
};

use async_recursion::async_recursion;
use rand::{seq::SliceRandom as _, thread_rng};
use std::time::Duration;
use tracing::{event, instrument, Level};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Root URLs is empty")]
    RootUrlsEmpty,
}

#[derive(Debug, thiserror::Error)]
pub enum CrawlWithRootUrlErrorKind {
    #[error(transparent)]
    Crawl(#[from] CrawlErrorKind),
    #[error("Depth limit reached")]
    DepthLimitReached,
    #[error("No URLs found")]
    NoUrlsFound,
}

pub struct Polling {
    client: Reqwest,
    route: Route,
    polling: PollingRules,
}

impl Polling {
    pub const fn new(client: Reqwest, route: Route, polling: PollingRules) -> Self {
        Self {
            client,
            route,
            polling,
        }
    }

    fn get_root_urls(&self) -> &RootUrls {
        &self.route.root_urls
    }

    const fn get_crawler(&self) -> Crawler {
        Crawler::new(&self.client, &self.route)
    }

    fn depth_matches(&self, depth: u16) -> bool {
        self.polling.depth_matches(depth)
    }

    fn get_random_sleep_between_requests(&self) -> Duration {
        self.polling.time.get_random_sleep_between_requests()
    }

    #[instrument(skip_all, fields(%url, %depth))]
    #[async_recursion]
    async fn run_with_parent_url(
        &self,
        url: &Url,
        depth: u16,
    ) -> Result<(), CrawlWithRootUrlErrorKind> {
        event!(Level::INFO, "Start crawling");

        if depth > 0 {
            if !self.depth_matches(depth) {
                return Err(CrawlWithRootUrlErrorKind::DepthLimitReached);
            }

            let sleep_duration = self.get_random_sleep_between_requests();

            event!(
                Level::INFO,
                "Sleeping for {:.2} ms",
                sleep_duration.as_millis(),
            );

            tokio::time::sleep(sleep_duration).await;
        }

        let mut urls = match self.get_crawler().crawl(url).await?.get_page_urls() {
            Some(urls) => urls.collect::<Vec<_>>().into_boxed_slice(),
            None => {
                return Err(CrawlWithRootUrlErrorKind::NoUrlsFound);
            }
        };

        let urls_len = urls.len();

        if urls_len > 1 {
            event!(Level::INFO, "Found {} URLs", urls_len);

            event!(
                Level::TRACE,
                "URLs: {}",
                urls.iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            event!(Level::INFO, "URLs not found");

            return Err(CrawlWithRootUrlErrorKind::NoUrlsFound);
        }

        urls.shuffle(&mut thread_rng());

        for url in &*urls {
            match self.run_with_parent_url(url, depth + 1).await {
                Ok(()) => {
                    // We don't want logging recursively similar logs that differ from the last one only by the depth
                    event!(
                        Level::TRACE,
                        child_url = %url,
                        "Crawling finished for child URL",
                    );

                    // We don't wabt to crawl all site URLs over and over again.
                    // So we stop crawling child URLs if we reached the depth limit at least once.
                    break;
                }
                Err(err) => match err {
                    CrawlWithRootUrlErrorKind::Crawl(err) => {
                        event!(Level::ERROR, %err, child_url = %url, "Error while crawling child URL");
                    }
                    CrawlWithRootUrlErrorKind::DepthLimitReached => {
                        event!(Level::WARN, child_url = %url, "Depth limit reached for child URL. Stop crawling child URLs and continue with next root URL");

                        break;
                    }
                    CrawlWithRootUrlErrorKind::NoUrlsFound => {
                        event!(Level::WARN, child_url = %url, "No URLs found for child URL");
                    }
                },
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn run(&self) -> Result<(), ErrorKind> {
        let root_urls = self.get_root_urls();

        if root_urls.is_empty() {
            return Err(ErrorKind::RootUrlsEmpty);
        }

        event!(
            Level::INFO,
            "Start polling with {} root URLs: {root_urls}",
            root_urls.len(),
        );

        loop {
            // `unwrap` is safe here because we checked that `root_urls` is not empty
            let root_url = root_urls.get_random().expect("Root URLs is empty");

            event!(Level::INFO, %root_url, "Start crawling with root URL");

            match self.run_with_parent_url(root_url, 0).await {
                Ok(()) => {
                    event!(Level::INFO, "Crawling finished");
                }
                Err(err) => match err {
                    CrawlWithRootUrlErrorKind::Crawl(err) => {
                        event!(Level::ERROR, %err, "Error while crawling root URL");
                    }
                    CrawlWithRootUrlErrorKind::DepthLimitReached => {
                        unreachable!("Depth limit reached for root URL, but it should never happen. Please, report this bug to the developers")
                    }
                    CrawlWithRootUrlErrorKind::NoUrlsFound => {
                        event!(
                            Level::WARN,
                            "No URLs found for root URL. Maybe you need to change the root URL or the crawler rules",
                        );
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rules::Rules;

    #[tokio::test]
    #[should_panic = "called `Result::unwrap()` on an `Err` value: RootUrlsEmpty"]
    async fn test_polling_panic() {
        let client = Reqwest::default().unwrap();
        let rules = Rules::default();

        let polling = Polling::new(client, rules.route, rules.polling);

        polling.run().await.unwrap();
    }
}
