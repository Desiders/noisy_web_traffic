use log::debug;
use reqwest::{blocking::Response as ReqwResponse, Error as ReqwError};
use serde_derive::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct BlacklistUrls {
    pub roots: Vec<String>,
    pub childs: Vec<String>,
    pub hrefs: Vec<String>,
    pub types: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MachineConfig {
    pub blacklist: BlacklistUrls,
}

pub fn create_config(path: &str) -> Result<(), Box<dyn Error>> {
    let config = MachineConfig {
        blacklist: BlacklistUrls {
            roots: vec![],
            childs: vec![],
            hrefs: vec![],
            types: vec![],
        },
    };
    let json = serde_json::to_string_pretty(&config)?;

    File::create(Path::new(path))?.write_all(json.as_bytes())?;
    debug!("Created machine config in `{}`", path);

    Ok(())
}

pub fn parse_config(path: &str) -> Result<MachineConfig, Box<dyn Error>> {
    let mut contents = String::new();

    File::open(Path::new(path))?.read_to_string(&mut contents)?;

    Ok(serde_json::from_str::<MachineConfig>(&contents)?)
}

pub fn write_blacklist_urls(
    path: &str,
    roots: &[String],
    childs: &[String],
    hrefs: &[String],
    types: &[String],
) -> Result<MachineConfig, Box<dyn Error>> {
    let mut config = parse_config(path)?;

    for (urls, save) in [
        (roots, &mut config.blacklist.roots),
        (childs, &mut config.blacklist.childs),
        (hrefs, &mut config.blacklist.hrefs),
        (types, &mut config.blacklist.types),
    ] {
        for url in urls {
            if !save.contains(url) {
                save.push(url.clone());
            }
        }
    }
    let json = serde_json::to_string_pretty(&config)?;

    File::options()
        .write(true)
        .truncate(true)
        .open(&Path::new(path))?
        .write_all(json.as_bytes())?;

    Ok(config)
}

pub fn write_blacklist_url_if_need(
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
    } else if let Some(err) = error {
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
