use crate::models::route::Route;

use tracing::{event, instrument, Level};
use url::Url;

#[instrument(skip_all, fields(%url))]
pub fn validate_url(url: &Url, route: &Route) -> bool {
    let Some(host) = url.host_str() else {
        event!(Level::TRACE, "No host found");

        return false;
    };
    let Some(port) = url.port_or_known_default() else {
        event!(Level::TRACE, "No port found");

        return false;
    };

    let scheme_matches = route.scheme_matches(url.scheme());
    let host_matches = route.host_matches(host);
    let port_matches = route.port_matches(port);
    let path_matches = route.path_matches(url.path());

    event!(
        Level::TRACE,
        scheme = scheme_matches,
        host = host_matches,
        port = port_matches,
        path = path_matches,
        "Match results",
    );

    scheme_matches && host_matches && port_matches && path_matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        let route = Route::default();

        let url = Url::parse("http://localhost:8080/").unwrap();

        assert!(validate_url(&url, &route));

        let url = Url::parse("http://localhost/").unwrap();

        assert!(validate_url(&url, &route));

        let url = Url::parse("test://localhost/").unwrap();

        assert!(!validate_url(&url, &route));

        let url = Url::parse("unix:/run/foo.socket").unwrap();

        assert!(!validate_url(&url, &route));

        let url = Url::parse("http://localhost:8080/foo").unwrap();

        assert!(validate_url(&url, &route));
    }
}
