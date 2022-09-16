use serde_derive::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

#[derive(Serialize, Deserialize)]
pub struct Client {
    pub max_depth: u32,
    pub min_sleep: u32,
    pub max_sleep: u32,
    pub max_timeout: u32,
    pub max_redirections: u32,
    pub max_failures: u32,
}

#[derive(Serialize, Deserialize)]
pub struct BlacklistUrls {
    pub childs: Vec<String>,
    pub hrefs: Vec<String>,
    pub types: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Urls {
    pub roots: Vec<String>,
    pub blacklist: BlacklistUrls,
}

#[derive(Serialize, Deserialize)]
pub struct UserAgent {
    pub generate: bool,
    pub current: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MachineConfig {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub client: Client,
    pub urls: Urls,
    pub user_agent: UserAgent,
    pub machine_config: MachineConfig,
}

pub fn parse_config(path: &str) -> serde_yaml::Result<Config> {
    let mut contents = String::new();

    File::open(Path::new(path))
        .expect("Failed to open config")
        .read_to_string(&mut contents)
        .expect("Failed to read config");

    serde_yaml::from_str::<Config>(&contents)
}
