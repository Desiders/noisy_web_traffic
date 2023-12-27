mod clients;
mod config;
mod crawlers;
mod models;
mod parser;
mod polling;
mod validation;

use clients::reqwest::Reqwest;
use config::parser::parse_rules_from_toml_file;
use polling::Polling;

use std::error::Error;
use tracing::{event, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("RUST_LOG"))
        .init();

    event!(Level::INFO, "Starting up");

    let route_config_path = "./config/route.toml";
    let polling_config_path = "./config/polling.toml";

    event!(
        Level::INFO,
        %route_config_path,
        %polling_config_path,
        "Parsing rules"
    );

    let rules = parse_rules_from_toml_file(route_config_path, polling_config_path)?;

    event!(Level::INFO, %rules, "Rules parsed");

    let client = Reqwest::default();

    let polling = Polling::new(client, rules.route, rules.polling);

    event!(Level::INFO, "Starting polling");

    polling.run().await?;

    unreachable!("Polling should never stop without an error")
}
