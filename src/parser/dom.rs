use tl::{HTMLTag as Tag, Node, ParseError, ParserOptions, VDom as Dom, VDomGuard as DomGuard};

#[allow(clippy::module_name_repetitions, dead_code)]
pub fn get_dom(raw: &str) -> Result<Dom<'_>, ParseError> {
    let dom = tl::parse(raw, ParserOptions::default())?;

    Ok(dom)
}

/// # Safety
/// This uses `unsafe` code to create a self-referential-like struct.
/// The given input string is first leaked and turned into raw pointer, and its lifetime will be promoted to 'static.
/// Once [`DomGuard`] goes out of scope, the string will be freed.
/// It should not be possible to cause UB in its current form.
pub fn get_dom_guard_with_options(
    raw: String,
    options: ParserOptions,
) -> Result<DomGuard, ParseError> {
    let dom = unsafe { tl::parse_owned(raw, options)? };

    Ok(dom)
}

/// # Safety
/// This uses `unsafe` code to create a self-referential-like struct.
/// The given input string is first leaked and turned into raw pointer, and its lifetime will be promoted to 'static.
/// Once [`DomGuard`] goes out of scope, the string will be freed.
/// It should not be possible to cause UB in its current form.
pub fn get_dom_guard(raw: String) -> Result<DomGuard, ParseError> {
    get_dom_guard_with_options(raw, ParserOptions::default())
}

pub(super) fn get_nodes<'s: 'dom, 'dom: 'dref, 'dref>(
    dom: &'dref Dom<'dom>,
    selector: &'s str,
) -> Option<impl Iterator<Item = &'dref Node<'dom>>> {
    let parser = dom.parser();

    dom.query_selector(selector)
        .map(|selectors| selectors.filter_map(|node| node.get(parser)))
}

pub(super) fn get_tags<'s: 'dom, 'dom: 'dref, 'dref>(
    dom: &'dref Dom<'dom>,
    selector: &'s str,
) -> Option<impl Iterator<Item = &'dref Tag<'dom>>> {
    get_nodes(dom, selector).map(|nodes| Iterator::filter_map(nodes, Node::as_tag))
}

pub(super) fn get_a_hrefs<'dom: 'dref, 'dref>(
    dom: &'dref Dom<'dom>,
) -> Option<impl Iterator<Item = &'dref str>> {
    get_tags(dom, "a[href]").map(|tags| {
        tags.filter_map(|tag| {
            tag.attributes()
                .get("href")
                .expect("href attribute cannot be empty, because of the selector")
        })
        .map(|href| href.try_as_utf8_str().unwrap())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom() {
        let dom = get_dom("<html><body><p>hello</p></body></html>").unwrap();
        let first_tag = dom.children()[0]
            .get(dom.parser())
            .unwrap()
            .as_tag()
            .unwrap();
        let first_tag_raw = first_tag.raw().try_as_utf8_str().unwrap();

        assert_eq!(first_tag_raw, "<html><body><p>hello</p></body></html>");
    }

    #[test]
    fn test_get_nodes() {
        let dom = get_dom(
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

        for node in get_nodes(&dom, "p").unwrap() {
            let tag = node.as_tag().unwrap();

            assert_eq!(tag.name(), "p");

            count += 1;
        }

        assert_eq!(count, 3);

        assert!(get_nodes(&dom, "div").unwrap().count() == 0);

        let mut count = 0;

        for node in get_nodes(&dom, "body").unwrap() {
            let tag = node.as_tag().unwrap();

            assert_eq!(tag.name(), "body");

            count += 1;
        }

        assert_eq!(count, 1);

        let dom = get_dom(
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

        for node in get_nodes(&dom, "a[href]").unwrap() {
            let tag = node.as_tag().unwrap();

            assert_eq!(tag.name(), "a");
            assert!(!tag.attributes().is_empty());

            count += 1;
        }

        assert_eq!(count, 3);
    }

    #[test]
    fn test_get_tags() {
        let dom = get_dom(
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

        for tag in get_tags(&dom, "p").unwrap() {
            assert_eq!(tag.name(), "p");

            count += 1;
        }

        assert_eq!(count, 3);

        assert!(get_tags(&dom, "div").unwrap().count() == 0);

        let mut count = 0;

        for tag in get_tags(&dom, "body").unwrap() {
            assert_eq!(tag.name(), "body");

            count += 1;
        }

        assert_eq!(count, 1);

        let dom = get_dom(
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

        for tag in get_tags(&dom, "a[href]").unwrap() {
            assert_eq!(tag.name(), "a");
            assert!(!tag.attributes().is_empty());

            count += 1;
        }

        assert_eq!(count, 3);
    }

    #[test]
    fn test_get_a_hrefs() {
        let dom = get_dom(
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

        for href in get_a_hrefs(&dom).unwrap() {
            assert_eq!(href, "https://example.com");

            count += 1;
        }

        assert_eq!(count, 3);
    }
}
