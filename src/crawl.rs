use crate::{
    client::Client,
    config_reader::Config,
    machine_config::{parse_config, write_blacklist_urls, MachineConfig},
    parser::{get_hrefs, get_url_from_href, parse_dom},
};
use log::{debug, warn};
use rand::{distributions::Uniform, seq::SliceRandom, thread_rng, Rng};
use reqwest::{blocking::Response as ReqwResponse, Error as ReqwError};
use std::{error::Error, thread::sleep as thread_sleep, time::Duration};

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
        debug!("Sleeping for {} seconds before crawling new one", time);
        thread_sleep(Duration::from_secs(u64::from(time)));
    }

    let resp = match client.get(url) {
        Ok(resp) => {
            if write_blacklist_urls_if_need(
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

            write_blacklist_urls_if_need(
                None,
                Some(&err),
                machine_config_path,
                url,
                current_depth == 0,
            )?;

            return Ok(CrawlResult::Failure);
        }
    };

    let html = resp.text()?;
    let dom = parse_dom(&html)?;
    let mut hrefs = get_hrefs(
        &dom,
        &machine_config.blacklist.hrefs,
        &machine_config.blacklist.types,
    );
    hrefs.shuffle(&mut thread_rng());

    let mut result = CrawlResult::Failure;
    let mut failure_urls = Vec::new();
    for href in hrefs {
        let url = match get_url_from_href(url, href, &machine_config.blacklist.childs) {
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
                failure_urls.push(url);
            }
        }
    }
    if !failure_urls.is_empty() {
        write_blacklist_urls(machine_config_path, &[], &failure_urls, &[], &[])?;
    }

    Ok(result)
}

fn write_blacklist_urls_if_need(
    response: Option<&ReqwResponse>,
    error: Option<&ReqwError>,
    machine_config_path: &str,
    url: &str,
    is_root_url: bool,
) -> Result<bool, Box<dyn Error>> {
    if let Some(resp) = response {
        if resp.status().is_success() {
            return Ok(false);
        }
    }
    if let Some(err) = error {
        if let Some(status) = err.status() {
            if !status.is_server_error() || !status.is_redirection() {
                return Ok(false);
            }
        } else if err.is_timeout() {
            return Ok(false);
        }
    }

    if is_root_url {
        write_blacklist_urls(machine_config_path, &[url.to_string()], &[], &[], &[])?;
    } else {
        write_blacklist_urls(machine_config_path, &[], &[url.to_string()], &[], &[])?;
    }
    debug!("Blacklisted url: `{}`", url);

    Ok(true)
}
