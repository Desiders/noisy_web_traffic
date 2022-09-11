use lazy_static::lazy_static;
use log::debug;
use regex::{Match, Regex};
use tl::{parse as parse_vdom, HTMLTag, ParseError, ParserOptions, VDom};

pub fn parse_dom(text: &str) -> Result<VDom, ParseError> {
    parse_vdom(text, ParserOptions::default())
}

pub fn get_hrefs<'a>(
    dom: &'a VDom,
    blacklist_hrefs: &[String],
    blacklist_types: &[String],
) -> Vec<&'a str> {
    let mut hrefs = Vec::new();

    let tags = get_tags(dom, "a[href]");
    debug!("Found {} tags in the tree", tags.len());

    for tag in tags {
        if let Some(href) = get_href_in_tag(tag) {
            if let Some(media_type_or_domain_match) = get_href_media_type_or_domain_match(href) {
                if let Some(media_type) =
                    get_href_media_type_in_match(href, &media_type_or_domain_match)
                {
                    // No need to strip suffix, it's done regex
                    if !blacklist_types.contains(&media_type.to_string()) {
                        hrefs.push(href);
                    }
                }
            } else if !value_in_blacklist(href, blacklist_hrefs) {
                hrefs.push(href);
            }
        }
    }

    hrefs
}

fn get_tags<'a>(dom: &'a VDom, selector: &str) -> Vec<&'a HTMLTag<'a>> {
    let mut tags = Vec::new();

    let dom_parser = dom.parser();
    if let Some(selector_iter) = dom.query_selector(selector) {
        selector_iter.for_each(|node_handle| {
            if let Some(node) = node_handle.get(dom_parser) {
                if let Some(tag) = node.as_tag() {
                    tags.push(tag);
                }
            }
        });
    }

    tags
}

fn get_href_in_tag<'a>(tag: &'a HTMLTag) -> Option<&'a str> {
    lazy_static! {
        static ref HREF: Regex = Regex::new(r"^(https?:/{2}|/\w+)\S*").unwrap(); // ^(https?:\/{2}|\/\w+)\S*
    }

    if let Some(Some(value)) = tag.attributes().get("href") {
        let string = value.try_as_utf8_str().unwrap();

        if !HREF.is_match(string) {
            return None;
        }
        return Some(string);
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

fn get_href_media_type_in_match<'a>(
    href: &str,
    media_type_or_domain_match: &'a Match,
) -> Option<&'a str> {
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
    let url = get_url_from_href(parent_url, href);

    if !value_in_blacklist(&url, blacklist_urls) {
        return Some(url);
    }
    None
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

fn value_in_blacklist(value: &str, blacklist: &[String]) -> bool {
    lazy_static! {
        static ref ARGUMENTS: Regex = Regex::new(r"(\?\S*)").unwrap(); // (\?\S*)
    }

    if let Some(cap) = ARGUMENTS.captures(value) {
        let arguments = cap.get(1).unwrap().as_str();
        let value_without_args = value.replace(arguments, "");

        let value_stripped = strip_value_after_slash(&value_without_args);
        for blacklist_value in blacklist {
            if value_stripped.starts_with(blacklist_value) {
                return true;
            }
        }
        return false;
    }

    let value_stripped = strip_value_after_slash(value);
    for blacklist_value in blacklist {
        if value_stripped.starts_with(blacklist_value) {
            return true;
        }
    }
    false
}

// for proper verification in the blacklist
fn strip_value_after_slash(value: &str) -> String {
    if value.ends_with('/') {
        value.rsplit_once('/').unwrap().0.to_string()
    } else {
        value.to_string()
    }
}
