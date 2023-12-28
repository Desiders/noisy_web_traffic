use crate::{
    clients::reqwest::Reqwest,
    models::route::Route,
    parser::{
        dom::get_dom_guard,
        robots_txt::{get_robot_rules, InvalidRobotRules},
        urls::get_urls_from_dom,
    },
    validation::route::validate_url,
};

use texting_robots::Robot;
use tl::VDomGuard as DomGuard;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum CrawlUrlErrorKind {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Parse(#[from] tl::ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum CrawlRobotsTxtErrorKind {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Parse(#[from] InvalidRobotRules),
}

pub struct Crawler<'a, 'b> {
    client: &'a Reqwest,
    route: &'b Route,
}

impl<'a, 'b> Crawler<'a, 'b> {
    pub const fn new(client: &'a Reqwest, route: &'b Route) -> Self {
        Self { client, route }
    }

    pub async fn crawl_url(&self, url: &Url) -> Result<CrawlerInner, CrawlUrlErrorKind> {
        let raw_html = self.client.get(url).await?.text().await?;
        let dom_guard = get_dom_guard(raw_html)?;

        Ok(CrawlerInner::new(dom_guard, self.route))
    }

    pub async fn crawl_robots_text(&self, url: &Url) -> Result<Robot, CrawlRobotsTxtErrorKind> {
        let raw_text = self.client.get(url).await?.text().await?;

        get_robot_rules(&self.client.user_agent(), &raw_text).map_err(Into::into)
    }
}

pub struct CrawlerInner<'a> {
    dom_guard: DomGuard,
    route: &'a Route,
}

impl<'a> CrawlerInner<'a> {
    pub const fn new(dom_guard: DomGuard, route: &'a Route) -> Self {
        Self { dom_guard, route }
    }

    pub fn get_page_urls(&self) -> Option<impl Iterator<Item = Url> + '_> {
        get_urls_from_dom(self.dom_guard.get_ref())
            .map(|urls| urls.filter(|url| validate_url(url, self.route)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rules::Rules;

    #[test]
    fn test_get_page_urls() {
        let dom = get_dom_guard(
            r#"
            <html>
                <body>
                    <a href="https://example1.com">hello</a>
                    <a href="https://example2.com">hello2</a>
                    <a href="https://example3.com">hello3</a>
                    <a href="hdata:text/plain,Stuff">hello4</a>
                    <a href="example5.com">hello5</a>
                    <a href="test://example6.com">hello6</a>
                    <a>hello5</a>
                    <link href="https://example.com" />
                </body>
            </html>"#
                .to_owned(),
        )
        .unwrap();

        let rules = Rules::default();

        let crawler = CrawlerInner::new(dom, &rules.route);

        let urls = crawler.get_page_urls().unwrap().collect::<Vec<_>>();

        assert_eq!(
            urls,
            [
                Url::parse("https://example1.com").unwrap(),
                Url::parse("https://example2.com").unwrap(),
                Url::parse("https://example3.com").unwrap(),
            ]
        );
    }
}
