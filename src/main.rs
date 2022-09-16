mod client;
mod config_reader;
mod crawl;
mod logger;
mod machine_config;
mod parser;

use client::Client;
use config_reader::parse_config;
use log::info;
use machine_config::{create_config, write_blacklist_urls};
use rand::{seq::SliceRandom, thread_rng};

fn main() {
    logger::init();

    let config = parse_config("./config.yaml").expect("Failed to parse config");

    let machine_config_path = format!("./{}.json", config.machine_config.name);

    create_config(&machine_config_path).expect("Failed to create machine config");

    write_blacklist_urls(
        &machine_config_path,
        &[],
        &config.urls.blacklist.childs,
        &config.urls.blacklist.hrefs,
        &config.urls.blacklist.types,
    )
    .expect("Failed to write blacklist URLs");

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

        crawl::run(&client, &config, &roots, &machine_config_path);
    }
}
