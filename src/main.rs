mod client;
mod config_reader;
mod crawl;
mod logger;
mod machine_config;
mod parser;

use client::Client;
use config_reader::parse_config;
use log::{error, info};
use machine_config::{create_config, write_blacklist_urls};
use rand::{seq::SliceRandom, thread_rng};

fn main() {
    logger::init();

    let config = match parse_config("./config.yaml") {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to parse config: {}", err);
            panic!("{}", err);
        }
    };

    let machine_config_path = format!("./{}.json", config.machine_config.name);

    if let Some(err) = create_config(&machine_config_path).err() {
        error!("Failed to create machine config: {}", err);
        panic!("{}", err)
    }

    if let Some(err) = write_blacklist_urls(
        &machine_config_path,
        &[],
        &config.urls.blacklist.childs,
        &config.urls.blacklist.hrefs,
        &config.urls.blacklist.types,
    )
    .err()
    {
        error!("Failed to write machine config: {}", err);
        panic!("{}", err);
    }

    let client = Client::new(
        config.client.max_timeout,
        config.client.max_redirections,
        &config.user_agent.current,
        config.user_agent.generate,
    );
    let mut roots = config.urls.roots.clone();

    info!("Starting crawl URLs");
    loop {
        roots.shuffle(&mut thread_rng());

        if let Some(err) = crawl::run(&client, &config, &roots, &machine_config_path).err() {
            error!("Failed to crawl: {}", err);
            panic!("{}", err);
        }
    }
}
