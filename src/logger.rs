use env_logger::{Builder, Env};

static DEFAULT_FILTER: &str = "info";

pub fn init() {
    Builder::from_env(Env::default().default_filter_or(DEFAULT_FILTER))
        .format_level(true)
        .format_module_path(true)
        .format_target(false)
        .format_indent(None)
        .format_timestamp_secs()
        .init();
}
