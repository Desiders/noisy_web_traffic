use super::dom::get_a_hrefs;

use tl::VDom as Dom;
use url::Url;

pub fn get_urls_from_dom<'dom: 'dref, 'dref>(
    dom: &'dref Dom<'dom>,
) -> Option<impl Iterator<Item = Url> + 'dref> {
    get_a_hrefs(dom).map(|hrefs| {
        hrefs
            .filter_map(|href| Url::parse(href).ok())
            .filter(Url::has_host) // https://url.spec.whatwg.org/#host-state
            .filter(Url::is_special) // https://url.spec.whatwg.org/#special-scheme
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::dom::get_dom;

    #[test]
    fn test_get_urls_from_dom() {
        let dom = get_dom(
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
            </html>"#,
        )
        .unwrap();

        let urls = get_urls_from_dom(&dom).unwrap().collect::<Vec<_>>();

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
