use std::{error::Error, fs::File, io::Read, path::Path};
use yaml_rust::YamlLoader;

pub struct Client {
    pub max_depth: u32,
    pub min_sleep: u32,
    pub max_sleep: u32,
    pub max_timeout: u32,
    pub max_redirections: u32,
    pub max_failures: u32,
}

pub struct BlacklistUrls {
    pub childs: Vec<String>,
    pub hrefs: Vec<String>,
    pub types: Vec<String>,
}

pub struct Urls {
    pub roots: Vec<String>,
    pub blacklist: BlacklistUrls,
}

pub struct UserAgent {
    pub generate: bool,
    pub current: Option<String>,
}

pub struct MachineConfig {
    pub name: String,
}

pub struct Config {
    pub client: Client,
    pub urls: Urls,
    pub user_agent: UserAgent,
    pub machine_config: MachineConfig,
}

pub fn parse_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let mut contents = String::new();

    File::open(&Path::new(path))?.read_to_string(&mut contents)?;

    let docs = YamlLoader::load_from_str(&contents)?;
    let doc = &docs[0];

    let client = {
        let client = &doc["client"];
        let max_depth = client["max_depth"].as_i64().unwrap().try_into()?;
        let min_sleep = client["min_sleep"].as_i64().unwrap().try_into()?;
        let max_sleep = client["max_sleep"].as_i64().unwrap().try_into()?;
        let max_timeout = client["max_timeout"].as_i64().unwrap().try_into()?;
        let max_redirections = client["max_redirections"].as_i64().unwrap().try_into()?;
        let max_failures = client["max_failures"].as_i64().unwrap().try_into()?;
        Client {
            max_depth,
            min_sleep,
            max_sleep,
            max_timeout,
            max_redirections,
            max_failures,
        }
    };

    let urls = {
        let urls = &doc["urls"];
        let roots = urls["roots"]
            .as_vec()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|url| url.as_str().unwrap().to_string())
            .collect();
        let blacklist = {
            let blacklist = &urls["blacklist"];
            let childs = blacklist["childs"]
                .as_vec()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|url| url.as_str().unwrap().to_string())
                .collect();
            let hrefs = blacklist["hrefs"]
                .as_vec()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|url| url.as_str().unwrap().to_string())
                .collect();
            let types = blacklist["types"]
                .as_vec()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|url| url.as_str().unwrap().to_string())
                .collect();
            BlacklistUrls {
                childs,
                hrefs,
                types,
            }
        };
        Urls { roots, blacklist }
    };

    let user_agent = {
        let user_agent = &doc["user_agent"];
        let generate = user_agent["generate"].as_bool().unwrap();
        let current = user_agent["current"].as_str().map(ToString::to_string);
        UserAgent { generate, current }
    };

    let machine_config = {
        let machine_config = &doc["machine_config"];
        let name = machine_config["name"].as_str().unwrap().to_string();
        MachineConfig { name }
    };

    Ok(Config {
        client,
        urls,
        user_agent,
        machine_config,
    })
}
