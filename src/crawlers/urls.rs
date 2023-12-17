use crate::{
    clients::reqwest::Reqwest,
    models::rules::Rules,
    parser::{dom::get_dom_guard, urls::get_urls_from_dom},
    validation::route::validate_url,
};

use tl::VDomGuard as DomGuard;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ParsE(#[from] tl::ParseError),
}

#[derive(Debug)]
pub struct Crawler {
    client: Reqwest,
    rules: Rules,
}

impl Crawler {
    pub const fn new(client: Reqwest, rules: Rules) -> Self {
        Self { client, rules }
    }

    pub async fn crawl(&self, url: &Url) -> Result<CrawlerInner, ErrorKind> {
        let raw_html = self.client.get(url).await?.text().await?;
        let dom_guard = get_dom_guard(raw_html)?;

        Ok(CrawlerInner::new(dom_guard, &self.rules))
    }
}

pub struct CrawlerInner<'a> {
    dom_guard: DomGuard,
    rules: &'a Rules,
}

impl<'a> CrawlerInner<'a> {
    pub const fn new(dom_guard: DomGuard, rules: &'a Rules) -> Self {
        Self { dom_guard, rules }
    }

    pub fn get_page_urls(&self) -> Option<impl Iterator<Item = Url> + '_> {
        get_urls_from_dom(self.dom_guard.get_ref())
            .map(|urls| urls.filter(|url| validate_url(url, &self.rules.route)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let crawler = CrawlerInner::new(dom, &rules);

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
