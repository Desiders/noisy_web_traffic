use html_parser::{Dom, Element, Error as HtmlParserError, Node};
use lazy_static::lazy_static;
use log::debug;
use regex::{Match, Regex};

pub fn parse_dom(text: &str) -> Result<Dom, HtmlParserError> {
    Dom::parse(text)
}

pub fn get_hrefs<'a>(
    dom: &'a Dom,
    blacklist_hrefs: &[String],
    blacklist_types: &[String],
) -> Vec<&'a String> {
    let mut hrefs = Vec::new();

    let elements = get_elements(dom);
    debug!("Found {} elements in the tree", elements.len());

    for element in elements {
        if let Some(href) = get_href(element, blacklist_hrefs, blacklist_types) {
            hrefs.push(href);
        }
    }

    hrefs
}

fn get_elements(dom: &Dom) -> Vec<&Element> {
    let mut elements = Vec::new();

    for node in &dom.children {
        push_elements(node, &mut elements);
    }

    elements
}

fn push_elements<'a>(node: &'a Node, elements: &mut Vec<&'a Element>) {
    if let Some(element) = node.element() {
        elements.push(element);
        for node in &element.children {
            push_elements(node, elements);
        }
    }
}

fn get_href<'a>(
    element: &'a Element,
    blacklist_hrefs: &[String],
    blacklist_types: &[String],
) -> Option<&'a String> {
    if let Some(href) = get_href_in_element(element) {
        if let Some(media_type_or_domain_match) = get_href_media_type_or_domain_match(href) {
            if let Some(media_type) = get_href_media_type(href, &media_type_or_domain_match) {
                // No need to strip suffix, it's done regex
                if !blacklist_types.contains(&media_type.to_string()) {
                    return Some(href);
                }
            }
        } else if !blacklist_hrefs.contains(&strip_suffix_slash(href)) {
            return Some(href);
        }
    }
    None
}

fn get_href_in_element(element: &Element) -> Option<&String> {
    lazy_static! {
        static ref HREF: Regex = Regex::new(r"^(https?:/{2}|/\w+)\S*").unwrap(); // ^(https?:\/{2}|\/\w+)\S*
    }

    if let Some(Some(value)) = element.attributes.get("href") {
        if !HREF.is_match(value) {
            return None;
        }
        return Some(value);
    }
    None
}

fn get_href_media_type_or_domain_match(href: &str) -> Option<Match> {
    lazy_static! {
        static ref MEDIA_TYPE_OR_DOMAIN: Regex =
            Regex::new(r"\.([a-zA-Z]+(/)?$)").unwrap(); // \.([a-zA-Z]+(\/)?$)
    }

    MEDIA_TYPE_OR_DOMAIN
        .captures(href)
        .map(|cap| cap.get(1).unwrap())
}

fn get_href_media_type<'a>(href: &str, media_type_or_domain_match: &'a Match) -> Option<&'a str> {
    lazy_static! {
        static ref PROTOCOL: Regex = Regex::new(r"^(https?:/{2})").unwrap(); // ^(https?:\/{2})
    }

    if href.starts_with('/') {
        // relative link with `/`
        let media_type = media_type_or_domain_match.as_str();
        return Some(media_type);
    } else if !PROTOCOL.is_match(href) {
        // ignore relative link without `/`
        // it's impossible, because regex check it, but here for clarity
        unimplemented!();
    }
    // absolute link with protocol `http` or `https`
    let slash_count = if href.ends_with('/') {
        href.matches('/').count() - 1
    } else {
        href.matches('/').count()
    };
    if slash_count > 2 {
        // href has got slash more than 2 times (2 becuase `https://` has got 2 slashes)
        let media_type = media_type_or_domain_match.as_str();
        return Some(media_type);
    }
    // ignore, because it's a domain
    None
}

pub fn get_url(parent_url: &str, href: &str, blacklist_urls: &[String]) -> Option<String> {
    lazy_static! {
        static ref ARGUMENTS: Regex = Regex::new(r"(\?\S*)").unwrap(); // (\?\S*)
    }

    let url = get_url_from_href(parent_url, href);

    if let Some(cap) = ARGUMENTS.captures(href) {
        let arguments = cap.get(1).unwrap().as_str();
        let url_without_args = url.replace(arguments, "");

        if !blacklist_urls.contains(&strip_suffix_slash(&url_without_args)) {
            Some(url)
        } else {
            None
        }
    } else {
        if !blacklist_urls.contains(&strip_suffix_slash(&url)) {
            Some(url)
        } else {
            None
        }
    }
}

// for proper verification in the blacklist
fn strip_suffix_slash(value: &str) -> String {
    if value.ends_with('/') {
        value.rsplit_once('/').unwrap().0.to_string()
    } else {
        value.to_string()
    }
}

fn get_url_from_href(parent_url: &str, href: &str) -> String {
    if href.starts_with("http") {
        href.to_string()
    } else {
        concat_url_with_href(parent_url, href)
    }
}

fn concat_url_with_href(url: &str, href: &str) -> String {
    if url.ends_with('/') {
        if !href.starts_with('/') {
            // it's impossible, because regex check it, but here for clarity
            unimplemented!();
        }
        let mut string = href.to_string();
        string.remove(0);

        format!("{}{}", url, string)
    } else {
        if !href.starts_with('/') {
            // it's impossible, because regex check it, but here for clarity
            unimplemented!();
        }
        format!("{}{}", url, href)
    }
}
