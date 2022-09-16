use lazy_static::lazy_static;
use log::debug;
use regex::{Match, Regex};
use std::time::Instant;
use tl::{parse as parse_vdom, HTMLTag, ParseError, ParserOptions, VDom};

pub fn parse_dom(text: &str) -> Result<VDom, ParseError> {
    let now = Instant::now();
    let result = parse_vdom(text, ParserOptions::default());
    debug!("DOM parsing took {} seconds", now.elapsed().as_secs_f32());

    result
}

pub fn get_hrefs<'a>(
    dom: &'a VDom,
    blacklist_hrefs: &[String],
    blacklist_types: &[String],
) -> Vec<&'a str> {
    let mut hrefs = vec![];

    let now = Instant::now();
    let tags = get_tags(dom, "a[href]");
    debug!("Found {} tags in the tree", tags.len());
    debug!("Getting tags took {} seconds", now.elapsed().as_secs_f32());

    let now = Instant::now();
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
    debug!("Found {} hrefs in the tree", hrefs.len());
    debug!("Getting hrefs took {} seconds", now.elapsed().as_secs_f32());

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

        if HREF.is_match(string) {
            Some(string)
        } else {
            None
        }
    } else {
        None
    }
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
        Some(media_type)
    } else {
        // ignore, because it's a domain
        None
    }
}

pub fn get_url(parent_url: &str, href: &str, blacklist_urls: &[String]) -> Option<String> {
    let url = get_url_from_href(parent_url, href);

    if value_in_blacklist(&url, blacklist_urls) {
        None
    } else {
        Some(url)
    }
}

fn get_url_from_href(parent_url: &str, href: &str) -> String {
    if href.starts_with('/') {
        concat_url_with_href(parent_url, href)
    } else {
        href.to_string()
    }
}

fn concat_url_with_href(url: &str, href: &str) -> String {
    lazy_static! {
        static ref PROTOCOL: Regex = Regex::new(r"^(https?:/{2})").unwrap(); // ^(https?:\/{2})
    }

    assert!(href.starts_with('/'));

    let cap = PROTOCOL.captures(url).unwrap();
    let protocol = cap.get(1).unwrap().as_str();
    let url_without_protocol = url.replace(protocol, "");

    let base_url = if let Some(slash_index) = url_without_protocol.find('/') {
        if url.rfind('/').unwrap() == slash_index {
            url_without_protocol.trim_end_matches('/').to_string()
        } else {
            url_without_protocol[..slash_index].to_string()
        }
    } else {
        url_without_protocol
    };

    debug!(
        "Protocol: {}, base_url: {}, href: {}",
        protocol, base_url, href
    );
    format!("{}{}{}", protocol, base_url, href)
}

pub fn value_in_blacklist(value: &str, blacklist: &[String]) -> bool {
    for blacklist_value in blacklist {
        if value.starts_with(blacklist_value) {
            return true;
        }
    }
    false
}
