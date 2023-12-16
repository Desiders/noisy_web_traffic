use std::borrow::Cow;
use tl::{Bytes, HTMLTag as Tag, Node, ParseError, ParserOptions, VDom as Dom};

pub fn dom(raw: &str, options: ParserOptions) -> Result<Dom<'_>, ParseError> {
    let dom = tl::parse(raw, options)?;

    Ok(dom)
}

pub fn dom_default(raw: &str) -> Result<Dom<'_>, ParseError> {
    dom(raw, ParserOptions::default())
}

pub(super) fn node_iter<'s: 'dom, 'dom>(
    dom: &'dom Dom<'dom>,
    selector: &'s str,
) -> Option<impl Iterator<Item = &'dom Node<'dom>> + 'dom> {
    let parser = dom.parser();

    dom.query_selector(selector)
        .map(|selector_iterator| selector_iterator.filter_map(move |node| node.get(&parser)))
}

pub(super) fn tag_iter<'s: 'dom, 'dom>(
    dom: &'dom Dom<'dom>,
    selector: &'s str,
) -> Option<impl Iterator<Item = &'dom Tag<'dom>> + 'dom> {
    node_iter(dom, selector).map(|node_iterator| Iterator::filter_map(node_iterator, Node::as_tag))
}

pub(super) fn a_href_iter<'s: 'dom, 'dom>(
    dom: &'dom Dom<'dom>,
) -> Option<impl Iterator<Item = Cow<'dom, str>> + 'dom> {
    tag_iter(dom, "a[href]").map(|tag_iterator| {
        tag_iterator
            .filter_map(|tag| {
                tag.attributes()
                    .get("href")
                    .expect("href attribute cannot be empty, because of the selector")
            })
            .map(|href| match href.try_as_utf8_str() {
                Some(href) => Cow::Borrowed(href),
                None => href.as_utf8_str(),
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom() {
        let dom = dom_default("<html><body><p>hello</p></body></html>").unwrap();
        let first_tag = dom.children()[0]
            .get(dom.parser())
            .unwrap()
            .as_tag()
            .unwrap();
        let first_tag_raw = first_tag.raw().try_as_utf8_str().unwrap();

        assert_eq!(first_tag_raw, "<html><body><p>hello</p></body></html>");
    }

    #[test]
    fn test_node_iter() {
        let dom = dom_default(
            r#"
            <html>
                <body>
                    <p>hello</p>
                    <p>hello2</p>
                    <p>hello3</p>
                </body>
            </html>"#,
        )
        .unwrap();

        let mut count = 0;

        for node in node_iter(&dom, "p").unwrap() {
            let tag = node.as_tag().unwrap();

            assert_eq!(tag.name(), "p");

            count += 1;
        }

        assert_eq!(count, 3);

        assert!(node_iter(&dom, "div").unwrap().count() == 0);

        let mut count = 0;

        for node in node_iter(&dom, "body").unwrap() {
            let tag = node.as_tag().unwrap();

            assert_eq!(tag.name(), "body");

            count += 1;
        }

        assert_eq!(count, 1);

        let dom = dom_default(
            r#"
            <html>
                <body>
                    <a href="https://example.com">hello</a>
                    <a href="https://example.com">hello2</a>
                    <a href="https://example.com">hello3</a>
                    <a>hello4</a>
                </body>
            </html>"#,
        )
        .unwrap();

        let mut count = 0;

        for node in node_iter(&dom, "a[href]").unwrap() {
            let tag = node.as_tag().unwrap();

            assert_eq!(tag.name(), "a");
            assert!(!tag.attributes().is_empty());

            count += 1;
        }

        assert_eq!(count, 3);
    }

    #[test]
    fn test_tag_iter() {
        let dom = dom_default(
            r#"
            <html>
                <body>
                    <p>hello</p>
                    <p>hello2</p>
                    <p>hello3</p>
                </body>
            </html>"#,
        )
        .unwrap();

        let mut count = 0;

        for tag in tag_iter(&dom, "p").unwrap() {
            assert_eq!(tag.name(), "p");

            count += 1;
        }

        assert_eq!(count, 3);

        assert!(tag_iter(&dom, "div").unwrap().count() == 0);

        let mut count = 0;

        for tag in tag_iter(&dom, "body").unwrap() {
            assert_eq!(tag.name(), "body");

            count += 1;
        }

        assert_eq!(count, 1);

        let dom = dom_default(
            r#"
            <html>
                <body>
                    <a href="https://example.com">hello</a>
                    <a href="https://example.com">hello2</a>
                    <a href="https://example.com">hello3</a>
                    <a>hello4</a>
                </body>
            </html>"#,
        )
        .unwrap();

        let mut count = 0;

        for tag in tag_iter(&dom, "a[href]").unwrap() {
            assert_eq!(tag.name(), "a");
            assert!(!tag.attributes().is_empty());

            count += 1;
        }

        assert_eq!(count, 3);
    }

    #[test]
    fn test_a_href_iter() {
        let dom = dom_default(
            r#"
            <html>
                <body>
                    <a href="https://example.com">hello</a>
                    <a href="https://example.com">hello2</a>
                    <a href="https://example.com">hello3</a>
                    <a>hello4</a>
                    <link href="https://example.com" />
                </body>
            </html>"#,
        )
        .unwrap();

        let mut count = 0;

        for href in a_href_iter(&dom).unwrap() {
            assert_eq!(href, "https://example.com");

            count += 1;
        }

        assert_eq!(count, 3);
    }
}
