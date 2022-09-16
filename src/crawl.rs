use crate::{
    client::Client,
    config_reader::Config,
    machine_config::{
        parse_config, write_blacklist_url_if_need, write_blacklist_urls, MachineConfig,
    },
    parser::{get_hrefs, get_url, parse_dom, value_in_blacklist},
};
use log::{debug, info};
use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};
use std::{
    thread::sleep as thread_sleep,
    time::{Duration, Instant},
};

enum CrawlResult {
    Success,
    Failure,
}

pub fn run(client: &Client, config: &Config, roots: &[String], machine_config_path: &str) {
    let machine_config = parse_config(machine_config_path).expect("Failed to parse machine config");

    let urls: Vec<&String> = roots
        .iter()
        .filter(|url| !value_in_blacklist(url, &machine_config.blacklist.roots))
        .collect();

    assert!(!urls.is_empty(), "Root URLs for crawling are empty");

    for url in urls {
        match crawl(client, config, &machine_config, machine_config_path, url, 0) {
            CrawlResult::Success => (),
            CrawlResult::Failure => info!("Failed to crawl the root URL: `{}`", url),
        }
    }
}

fn crawl(
    client: &Client,
    config: &Config,
    machine_config: &MachineConfig,
    machine_config_path: &str,
    url: &str,
    current_depth: u32,
) -> CrawlResult {
    if current_depth >= config.client.max_depth {
        info!("Maximum depth reached");

        return CrawlResult::Success;
    } else if current_depth > 0 {
        let time = thread_rng().sample(Uniform::new(
            config.client.min_sleep,
            config.client.max_sleep,
        ));
        debug!(
            "Sleeps for {} seconds before starting a new one. Current depth: {}",
            time, current_depth
        );
        thread_sleep(Duration::from_secs(u64::from(time)));
    }

    let resp = match client.get(url) {
        Ok(resp) => {
            if write_blacklist_url_if_need(
                Some(&resp),
                None,
                machine_config_path,
                url,
                current_depth == 0,
            )
            .expect("Failed to write blacklist URL")
            {
                info!("Failed to crawl URL `{}`", url);

                return CrawlResult::Failure;
            }
            resp
        }
        Err(err) => {
            info!("Failed to crawl URL `{}`: {}", url, err);

            write_blacklist_url_if_need(
                None,
                Some(&err),
                machine_config_path,
                url,
                current_depth == 0,
            )
            .expect("Failed to write blacklist URL");

            return CrawlResult::Failure;
        }
    };
    let new_url = resp.url().clone();

    let now = Instant::now();
    let html = match resp.text() {
        Ok(html) => html,
        Err(err) => {
            info!("Couldn't get HTML from URL `{}`: {}", url, err);
            return CrawlResult::Failure;
        }
    };
    debug!(
        "The HTML parsing took {} seconds. Length of text and lines: {}, {}",
        now.elapsed().as_secs_f32(),
        html.len(),
        html.lines().count(),
    );

    let dom = parse_dom(&html).expect("Failed to parse DOM");
    let mut hrefs = get_hrefs(
        &dom,
        &machine_config.blacklist.hrefs,
        &machine_config.blacklist.types,
    );
    if hrefs.is_empty() {
        return CrawlResult::Failure;
    }

    hrefs.shuffle(&mut thread_rng());

    let mut result = CrawlResult::Failure;
    let mut failure_urls = vec![];
    let mut failure_urls_len: u32 = 0;
    for href in hrefs {
        let url = match get_url(new_url.as_str(), href, &machine_config.blacklist.childs) {
            Some(url) => url,
            None => continue,
        };

        match crawl(
            client,
            config,
            machine_config,
            machine_config_path,
            &url,
            current_depth + 1,
        ) {
            CrawlResult::Success => {
                result = CrawlResult::Success;
                break;
            }
            CrawlResult::Failure => {
                if failure_urls_len > config.client.max_failures {
                    info!("Too many failures, stopped crawling `{}' child URLs", url);
                    break;
                }
                failure_urls.push(url);
                failure_urls_len += 1;
            }
        }
    }
    if !failure_urls.is_empty() {
        write_blacklist_urls(machine_config_path, &[], &failure_urls, &[], &[])
            .expect("Failed to write blacklist URLs");
    }

    result
}
