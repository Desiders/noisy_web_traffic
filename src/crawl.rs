use crate::{
    client::Client,
    config_reader::Config,
    machine_config::{
        parse_config, write_blacklist_url_if_need, write_blacklist_urls, MachineConfig,
    },
    parser::{get_hrefs, get_url, parse_dom},
};
use log::{debug, warn};
use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};
use std::{
    error::Error,
    thread::sleep as thread_sleep,
    time::{Duration, Instant},
};

enum CrawlResult {
    Success,
    Failure,
}

pub fn run(
    client: &Client,
    config: &Config,
    roots: &[String],
    machine_config_path: &str,
) -> Result<(), Box<dyn Error>> {
    let machine_config = parse_config(machine_config_path)?;

    for url in roots
        .iter()
        .filter(|url| !machine_config.blacklist.roots.contains(url))
    {
        match crawl(client, config, &machine_config, machine_config_path, url, 0)? {
            CrawlResult::Success => (),
            CrawlResult::Failure => warn!("Failed to crawl by root URL `{}`", url),
        }
    }

    Ok(())
}

fn crawl(
    client: &Client,
    config: &Config,
    machine_config: &MachineConfig,
    machine_config_path: &str,
    url: &str,
    current_depth: u32,
) -> Result<CrawlResult, Box<dyn Error>> {
    if current_depth >= config.client.max_depth {
        return Ok(CrawlResult::Success);
    } else if current_depth > 0 {
        let time = thread_rng().sample(Uniform::new(
            config.client.min_sleep,
            config.client.max_sleep,
        ));
        debug!(
            "Sleeping for {} seconds before crawling new one. Current depth: {}",
            time, current_depth
        );
        thread_sleep(Duration::from_secs(u64::from(time)));
    }

    let now = Instant::now();
    let resp = match client.get(url) {
        Ok(resp) => {
            if write_blacklist_url_if_need(
                Some(&resp),
                None,
                machine_config_path,
                url,
                current_depth == 0,
            )? {
                return Ok(CrawlResult::Failure);
            }
            resp
        }
        Err(err) => {
            debug!("Failed to crawl url `{}`: {}", url, err);

            write_blacklist_url_if_need(
                None,
                Some(&err),
                machine_config_path,
                url,
                current_depth == 0,
            )?;

            return Ok(CrawlResult::Failure);
        }
    };
    debug!("Crawled url took {} seconds", now.elapsed().as_secs_f32());

    let now = Instant::now();
    let html = match resp.text() {
        Ok(html) => html,
        Err(err) => {
            debug!("Failed to get html from url `{}`: {}", url, err);
            return Ok(CrawlResult::Failure);
        }
    };
    debug!(
        "Parsing HTML took {} seconds. Text length and lines: {}, {}",
        now.elapsed().as_secs_f32(),
        html.len(),
        html.lines().count(),
    );

    let now = Instant::now();
    let dom = parse_dom(&html)?;
    debug!("Parsing DOM took {} seconds", now.elapsed().as_secs_f32(),);

    let now = Instant::now();
    let mut hrefs = get_hrefs(
        &dom,
        &machine_config.blacklist.hrefs,
        &machine_config.blacklist.types,
    );
    debug!(
        "Getting hrefs took {} seconds. Hrefs count: {}",
        now.elapsed().as_secs_f32(),
        hrefs.len()
    );

    hrefs.shuffle(&mut thread_rng());

    let mut result = CrawlResult::Failure;
    let mut failure_urls = Vec::new();
    for href in hrefs {
        let url = match get_url(url, href, &machine_config.blacklist.childs) {
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
        )? {
            CrawlResult::Success => {
                result = CrawlResult::Success;
                break;
            }
            CrawlResult::Failure => {
                if failure_urls.len() > config.client.max_failures as usize {
                    debug!(
                        "Too many failures, stopped crawling to `{}`'s children URLs",
                        url
                    );
                    break;
                }
                failure_urls.push(url);
            }
        }
    }
    if !failure_urls.is_empty() {
        write_blacklist_urls(machine_config_path, &[], &failure_urls, &[], &[])?;
    }

    Ok(result)
}
