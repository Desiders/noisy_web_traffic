use texting_robots::Robot;

#[allow(clippy::module_name_repetitions)]
pub fn validate_robots_rules(url: impl AsRef<str>, robots: &Robot) -> bool {
    robots.allowed(url.as_ref())
}
