use crate::models::{
    polling::{depth, proxy, redirections, time, user_agent, Polling},
    route::Route,
    routes::{
        host, path,
        permission::Kind as PermissionKind,
        port, root_url,
        scheme::{self, UnsupportedSchemeError},
    },
    rules::Rules,
};

use glob::PatternError;
use std::{fs, io, num::ParseIntError, path::Path};
use toml::Value;
use tracing::{event, field, instrument, Level, Span};

#[derive(Debug, thiserror::Error)]
pub enum ParseRouteErrorKind {
    #[error("Parse toml error: {0}")]
    ParseToml(#[from] toml::de::Error),

    #[error("Routes not found: {0}")]
    RoutesNotFound(Value),
    #[error("Routes must be a table, found {0}")]
    RoutesMustBeTable(Value),

    #[error("Root urls must be an array, found {0}")]
    RootUrlsMustBeArray(Value),
    #[error("Root url value must be a string, found {0}")]
    RootUrlValueMustBeString(Value),
    #[error("Root url parse error: {0}")]
    RootUrlParseError(url::ParseError),

    #[error("Hosts must be an array, found {0}")]
    HostsMustBeArray(Value),
    #[error("Host value must be a string, found {0}")]
    HostExactMustBeString(Value),
    #[error("Host glob must be a string, found {0}")]
    HostGlobMustBeString(Value),
    #[error("Host glob pattern error: {0}")]
    HostGlobPattern(PatternError),
    #[error("Host parse error: {0}")]
    HostParseError(url::ParseError),

    #[error("Schemes must be an array, found {0}")]
    SchemesMustBeArray(Value),
    #[error("Scheme value must be a string, found {0}")]
    SchemeExactMustBeString(Value),
    #[error(transparent)]
    UnsupportedScheme(#[from] UnsupportedSchemeError),

    #[error("Ports must be an array, found {0}")]
    PortsMustBeArray(Value),
    #[error("Port glob must be a string, found {0}")]
    PortGlobMustBeString(Value),
    #[error("Port glob pattern error: {0}")]
    PortGlobPattern(PatternError),
    #[error("Port value parse error: {0}")]
    PortExactParseError(ParseIntError),
    #[error("Port value must be an int or a string that represents an int, found {0}")]
    PortExactMustBeStringOrInt(Value),

    #[error("Paths must be an array, found {0}")]
    PathsMustBeArray(Value),
    #[error("Path glob pattern error: {0}")]
    PathGlobPattern(PatternError),
    #[error("Path glob must be a string, found {0}")]
    PathGlobMustBeString(Value),
    #[error("Path value must be a string, found {0}")]
    PathExactMustBeString(Value),
}

/// Parse route from toml
/// # Arguments
/// * `raw` - Raw toml string
/// # Returns
/// Returns [`Route`] if parsing is successful and all routes are valid, otherwise returns [`ParseRouteErrorKind`].
/// # Panics
/// If the port number is not between 0 and 65535
#[instrument(skip_all)]
pub fn parse_route_from_toml(raw: &str) -> Result<Route, ParseRouteErrorKind> {
    event!(Level::DEBUG, "Parse route from toml");

    let value = raw.parse::<Value>()?;

    let routes = match value.get("routes") {
        Some(routes) => match routes.as_table() {
            Some(routes) => routes,
            None => return Err(ParseRouteErrorKind::RoutesMustBeTable(routes.clone())),
        },
        None => return Err(ParseRouteErrorKind::RoutesNotFound(value.clone())),
    };

    let mut route_builder = Route::builder();

    match routes.get("root_urls") {
        Some(root_urls) => {
            event!(Level::TRACE, "Parse root urls");

            let Some(root_urls) = root_urls.as_array() else {
                return Err(ParseRouteErrorKind::RootUrlsMustBeArray(root_urls.clone()));
            };

            for root_url in root_urls {
                if let Some(value) = root_url.get("value") {
                    match value.as_str() {
                        Some(root_url) => {
                            route_builder = route_builder.root_url(
                                root_url::RootUrl::new(root_url)
                                    .map_err(ParseRouteErrorKind::RootUrlParseError)?,
                            );
                        }
                        None => {
                            return Err(ParseRouteErrorKind::RootUrlValueMustBeString(
                                value.clone(),
                            ))
                        }
                    }
                }
            }
        }
        None => {
            event!(Level::TRACE, "Root urls not found");
        }
    }

    match routes.get("hosts") {
        Some(hosts) => {
            event!(Level::TRACE, "Parse hosts");

            match hosts.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable hosts");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ParseRouteErrorKind::HostsMustBeArray(acceptable.clone()));
                    };

                    for host in acceptable {
                        if let Some(glob) = host.get("glob") {
                            match glob.as_str() {
                                Some(host) => {
                                    route_builder = route_builder.host(host::Matcher::new(
                                        PermissionKind::Acceptable,
                                        host::Kind::glob(host)
                                            .map_err(ParseRouteErrorKind::HostGlobPattern)?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::HostGlobMustBeString(
                                        glob.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Host glob not found");

                        if let Some(exact) = host.get("exact") {
                            match exact.as_str() {
                                Some(host) => {
                                    route_builder = route_builder.host(host::Matcher::new(
                                        PermissionKind::Acceptable,
                                        host::Kind::exact(host)
                                            .map_err(ParseRouteErrorKind::HostParseError)?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::HostExactMustBeString(
                                        exact.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Host exact not found");
                    }
                }
                None => {
                    event!(Level::TRACE, "Acceptable hosts not found");
                }
            }

            match hosts.get("unacceptable") {
                Some(unacceptable) => {
                    event!(Level::TRACE, "Parse unacceptable hosts");

                    let Some(unacceptable) = unacceptable.as_array() else {
                        return Err(ParseRouteErrorKind::HostsMustBeArray(unacceptable.clone()));
                    };

                    for host in unacceptable {
                        if let Some(glob) = host.get("glob") {
                            match glob.as_str() {
                                Some(host) => {
                                    route_builder = route_builder.host(host::Matcher::new(
                                        PermissionKind::Unacceptable,
                                        host::Kind::glob(host)
                                            .map_err(ParseRouteErrorKind::HostGlobPattern)?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::HostGlobMustBeString(
                                        glob.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Host glob not found");

                        if let Some(exact) = host.get("exact") {
                            match exact.as_str() {
                                Some(host) => {
                                    route_builder = route_builder.host(host::Matcher::new(
                                        PermissionKind::Unacceptable,
                                        host::Kind::exact(host)
                                            .map_err(ParseRouteErrorKind::HostParseError)?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::HostExactMustBeString(
                                        exact.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Host exact not found");
                    }
                }
                None => {
                    event!(Level::TRACE, "Unacceptable hosts not found");
                }
            }
        }
        None => {
            event!(Level::TRACE, "Hosts not found");
        }
    }

    match routes.get("schemes") {
        Some(schemes) => {
            event!(Level::TRACE, "Parse schemes");

            match schemes.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable schemes");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ParseRouteErrorKind::SchemesMustBeArray(acceptable.clone()));
                    };

                    for scheme in acceptable {
                        if let Some(exact) = scheme.get("exact") {
                            match exact.as_str() {
                                Some(scheme) => {
                                    route_builder = route_builder.scheme(scheme::Matcher::new(
                                        PermissionKind::Acceptable,
                                        scheme::Kind::try_from(scheme.to_owned())?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::SchemeExactMustBeString(
                                        exact.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Scheme exact not found");
                    }
                }
                None => {
                    event!(Level::TRACE, "Acceptable schemes not found");
                }
            }

            match schemes.get("unacceptable") {
                Some(unacceptable) => {
                    event!(Level::TRACE, "Parse unacceptable schemes");

                    let Some(unacceptable) = unacceptable.as_array() else {
                        return Err(ParseRouteErrorKind::SchemesMustBeArray(
                            unacceptable.clone(),
                        ));
                    };

                    for scheme in unacceptable {
                        if let Some(exact) = scheme.get("exact") {
                            match exact.as_str() {
                                Some(scheme) => {
                                    route_builder = route_builder.scheme(scheme::Matcher::new(
                                        PermissionKind::Unacceptable,
                                        scheme::Kind::try_from(scheme.to_owned())?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::SchemeExactMustBeString(
                                        exact.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Scheme exact not found");
                    }
                }
                None => {
                    event!(Level::TRACE, "Unacceptable schemes not found");
                }
            }
        }
        None => {
            event!(Level::TRACE, "Schemes not found");
        }
    }

    match routes.get("ports") {
        Some(ports) => {
            event!(Level::TRACE, "Parse ports");

            match ports.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable ports");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ParseRouteErrorKind::PortsMustBeArray(acceptable.clone()));
                    };

                    for port in acceptable {
                        if let Some(glob) = port.get("glob") {
                            match glob.as_str() {
                                Some(port) => {
                                    route_builder = route_builder.port(port::Matcher::new(
                                        PermissionKind::Acceptable,
                                        port::Kind::glob(port)
                                            .map_err(ParseRouteErrorKind::PortGlobPattern)?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::PortGlobMustBeString(
                                        glob.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Port glob not found");

                        if let Some(exact) = port.get("exact") {
                            if let Some(port) = exact.as_str() {
                                route_builder = route_builder.port(port::Matcher::new(
                                    PermissionKind::Acceptable,
                                    port::Kind::exact_str(port)
                                        .map_err(ParseRouteErrorKind::PortExactParseError)?,
                                ));

                                continue;
                            }

                            if let Some(port) = exact.as_integer() {
                                route_builder = route_builder.port(port::Matcher::new(
                                    PermissionKind::Acceptable,
                                    port::Kind::exact(
                                        u16::try_from(port)
                                            .expect("Port number must be between 0 and 65535"),
                                    ),
                                ));

                                continue;
                            }

                            return Err(ParseRouteErrorKind::PortExactMustBeStringOrInt(
                                exact.clone(),
                            ));
                        }

                        event!(Level::TRACE, "Port exact not found");
                    }
                }
                None => {
                    event!(Level::TRACE, "Acceptable ports not found");
                }
            }

            match ports.get("unacceptable") {
                Some(unacceptable) => {
                    event!(Level::TRACE, "Parse unacceptable ports");

                    let Some(unacceptable) = unacceptable.as_array() else {
                        return Err(ParseRouteErrorKind::PortsMustBeArray(unacceptable.clone()));
                    };

                    for port in unacceptable {
                        if let Some(glob) = port.get("glob") {
                            match glob.as_str() {
                                Some(port) => {
                                    route_builder = route_builder.port(port::Matcher::new(
                                        PermissionKind::Unacceptable,
                                        port::Kind::glob(port)
                                            .map_err(ParseRouteErrorKind::PortGlobPattern)?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::PortGlobMustBeString(
                                        glob.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Port glob not found");

                        if let Some(exact) = port.get("exact") {
                            if let Some(port) = exact.as_str() {
                                route_builder = route_builder.port(port::Matcher::new(
                                    PermissionKind::Unacceptable,
                                    port::Kind::exact_str(port)
                                        .map_err(ParseRouteErrorKind::PortExactParseError)?,
                                ));

                                continue;
                            }

                            if let Some(port) = exact.as_integer() {
                                route_builder = route_builder.port(port::Matcher::new(
                                    PermissionKind::Unacceptable,
                                    port::Kind::exact(
                                        u16::try_from(port)
                                            .expect("Port number must be between 0 and 65535"),
                                    ),
                                ));

                                continue;
                            }

                            return Err(ParseRouteErrorKind::PortExactMustBeStringOrInt(
                                exact.clone(),
                            ));
                        }

                        event!(Level::TRACE, "Port exact not found");
                    }
                }
                None => {
                    event!(Level::TRACE, "Unacceptable ports not found");
                }
            }
        }
        None => {
            event!(Level::TRACE, "Ports not found");
        }
    }

    match routes.get("paths") {
        Some(paths) => {
            event!(Level::TRACE, "Parse paths");

            match paths.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable paths");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ParseRouteErrorKind::PathsMustBeArray(acceptable.clone()));
                    };

                    for path in acceptable {
                        if let Some(glob) = path.get("glob") {
                            match glob.as_str() {
                                Some(path) => {
                                    route_builder = route_builder.path(path::Matcher::new(
                                        PermissionKind::Acceptable,
                                        path::Kind::glob(path)
                                            .map_err(ParseRouteErrorKind::PathGlobPattern)?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::PathGlobMustBeString(
                                        glob.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Path glob not found");

                        if let Some(exact) = path.get("exact") {
                            match exact.as_str() {
                                Some(path) => {
                                    route_builder = route_builder.path(path::Matcher::new(
                                        PermissionKind::Acceptable,
                                        path::Kind::exact(path),
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::PathExactMustBeString(
                                        exact.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Path exact not found");
                    }
                }
                None => {
                    event!(Level::TRACE, "Acceptable paths not found");
                }
            }

            match paths.get("unacceptable") {
                Some(unacceptable) => {
                    event!(Level::TRACE, "Parse unacceptable paths");

                    let Some(unacceptable) = unacceptable.as_array() else {
                        return Err(ParseRouteErrorKind::PathsMustBeArray(unacceptable.clone()));
                    };

                    for path in unacceptable {
                        if let Some(glob) = path.get("glob") {
                            match glob.as_str() {
                                Some(path) => {
                                    route_builder = route_builder.path(path::Matcher::new(
                                        PermissionKind::Unacceptable,
                                        path::Kind::glob(path)
                                            .map_err(ParseRouteErrorKind::PathGlobPattern)?,
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::PathGlobMustBeString(
                                        glob.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Path glob not found");

                        if let Some(exact) = path.get("exact") {
                            match exact.as_str() {
                                Some(path) => {
                                    route_builder = route_builder.path(path::Matcher::new(
                                        PermissionKind::Unacceptable,
                                        path::Kind::exact(path),
                                    ));

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::PathExactMustBeString(
                                        exact.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Path exact not found");
                    }
                }
                None => {
                    event!(Level::TRACE, "Unacceptable paths not found");
                }
            }
        }
        None => {
            event!(Level::TRACE, "Paths not found");
        }
    }

    Ok(route_builder.build())
}

#[derive(Debug, thiserror::Error)]
pub enum ParsePollingErrorKind {
    #[error("Parse toml error: {0}")]
    ParseToml(#[from] toml::de::Error),

    #[error("Polling not found: {0}")]
    PollingNotFound(Value),
    #[error("Polling must be a table, found {0}")]
    PollingMustBeTable(Value),

    #[error("Redirections acceptable not found: {0}")]
    RedirectionsAcceptableNotFound(Value),
    #[error("Redirections acceptable must be a bool, found {0}")]
    RedirectionsAcceptableMustBeBool(Value),
    #[error("Redirections max redirects not found: {0}")]
    RedirectionsMaxRedirectsNotFound(Value),
    #[error(
        "Redirections max redirects must be an int or a string that represents an int, found {0}"
    )]
    RedirectionsMaxRedirectsMustBeStringOrInt(Value),

    #[error("Depth acceptable not found: {0}")]
    DepthAcceptableNotFound(Value),
    #[error("Depth acceptable must be a bool, found {0}")]
    DepthAcceptableMustBeBool(Value),
    #[error("Depth max redirects not found: {0}")]
    DepthMaxRedirectsNotFound(Value),
    #[error("Depth max depth must be an int or a string that represents an int, found {0}")]
    DepthMaxDepthMustBeStringOrInt(Value),

    #[error("Min sleep between requests not found: {0}")]
    MinSleepBetweenRequestsNotFound(Value),
    #[error("Min sleep between requests value must be an int or a string that represents an int, found {0}")]
    MinSleepBetweenRequestsMustBeStringOrInt(Value),
    #[error("Max sleep between requests not found: {0}")]
    MaxSleepBetweenRequestsNotFound(Value),
    #[error("Max sleep between requests value must be an int or a string that represents an int, found {0}")]
    MaxSleepBetweenRequestsMustBeStringOrInt(Value),
    #[error("Request timeout not found: {0}")]
    RequestTimeoutNotFound(Value),
    #[error("Request timeout value must be an int or a string that represents an int, found {0}")]
    RequestTimeoutMustBeStringOrInt(Value),

    #[error("User agent value must be a string, found {0}")]
    UserAgentValueMustBeString(Value),

    #[error("Proxy value must be a string, found {0}")]
    ProxyValueMustBeString(Value),
}

/// Parse polling from toml
/// # Arguments
/// * `raw` - Raw toml string
/// # Returns
/// Returns [`Polling`] if parsing is successful and all polling are valid, otherwise returns [`ParsePollingErrorKind`].
/// # Panics
/// - If the max redirects is not between 0 and 65535
/// - If the max depth is not between 0 and 65535
/// - If the min sleep between requests is not between 0 and 18446744073709551615
/// - If the max sleep between requests is not between 0 and 18446744073709551615
/// - If the request timeout is not between 0 and 18446744073709551615
#[instrument(skip_all)]
pub fn parse_polling_from_toml(raw: &str) -> Result<Polling, ParsePollingErrorKind> {
    event!(Level::DEBUG, "Parse polling from toml");

    let value = raw.parse::<Value>()?;

    let polling = match value.get("polling") {
        Some(polling) => match polling.as_table() {
            Some(polling) => polling,
            None => return Err(ParsePollingErrorKind::PollingMustBeTable(polling.clone())),
        },
        None => return Err(ParsePollingErrorKind::PollingNotFound(value.clone())),
    };

    let mut polling_builder = Polling::builder();

    match polling.get("redirections") {
        Some(redirections) => {
            event!(Level::TRACE, "Parse redirections");

            let acceptable = if let Some(acceptable) = redirections.get("acceptable") {
                if let Some(acceptable) = acceptable.as_bool() {
                    acceptable
                } else {
                    return Err(ParsePollingErrorKind::RedirectionsAcceptableMustBeBool(
                        acceptable.clone(),
                    ));
                }
            } else {
                return Err(ParsePollingErrorKind::RedirectionsAcceptableNotFound(
                    redirections.clone(),
                ));
            };

            let max_redirects = if let Some(max_redirects) = redirections.get("max_redirects") {
                if let Some(max_redirects_str) = max_redirects.as_str() {
                    max_redirects_str.parse::<u16>().map_err(|_| {
                        ParsePollingErrorKind::RedirectionsMaxRedirectsMustBeStringOrInt(
                            max_redirects.clone(),
                        )
                    })?
                } else if let Some(max_redirects) = max_redirects.as_integer() {
                    u16::try_from(max_redirects).expect("Max redirects must be between 0 and 65535")
                } else {
                    return Err(
                        ParsePollingErrorKind::RedirectionsMaxRedirectsMustBeStringOrInt(
                            max_redirects.clone(),
                        ),
                    );
                }
            } else {
                return Err(ParsePollingErrorKind::RedirectionsMaxRedirectsNotFound(
                    redirections.clone(),
                ));
            };

            polling_builder = polling_builder
                .redirections(redirections::Redirections::new(acceptable, max_redirects));
        }
        None => {
            event!(Level::TRACE, "Redirections not found");
        }
    }

    match polling.get("depth") {
        Some(depth) => {
            event!(Level::TRACE, "Parse depth");

            let acceptable = if let Some(acceptable) = depth.get("acceptable") {
                if let Some(acceptable) = acceptable.as_bool() {
                    acceptable
                } else {
                    return Err(ParsePollingErrorKind::DepthAcceptableMustBeBool(
                        acceptable.clone(),
                    ));
                }
            } else {
                return Err(ParsePollingErrorKind::DepthAcceptableNotFound(
                    depth.clone(),
                ));
            };

            let max_depth = if let Some(max_depth) = depth.get("max_depth") {
                if let Some(max_depth_str) = max_depth.as_str() {
                    max_depth_str.parse::<u16>().map_err(|_| {
                        ParsePollingErrorKind::DepthMaxDepthMustBeStringOrInt(max_depth.clone())
                    })?
                } else if let Some(max_depth) = max_depth.as_integer() {
                    u16::try_from(max_depth).expect("Max depth must be between 0 and 65535")
                } else {
                    return Err(ParsePollingErrorKind::DepthMaxDepthMustBeStringOrInt(
                        max_depth.clone(),
                    ));
                }
            } else {
                return Err(ParsePollingErrorKind::DepthMaxRedirectsNotFound(
                    depth.clone(),
                ));
            };

            polling_builder = polling_builder.depth(depth::Depth::new(acceptable, max_depth));
        }
        None => {
            event!(Level::TRACE, "Depth not found");
        }
    }

    match polling.get("time") {
        Some(time) => {
            event!(Level::TRACE, "Parse time");

            let min_sleep_between_requests = if let Some(min_sleep_between_requests) =
                time.get("min_sleep_between_requests")
            {
                if let Some(min_sleep_between_requests_str) = min_sleep_between_requests.as_str() {
                    min_sleep_between_requests_str.parse::<u64>().map_err(|_| {
                        ParsePollingErrorKind::MinSleepBetweenRequestsMustBeStringOrInt(
                            min_sleep_between_requests.clone(),
                        )
                    })?
                } else if let Some(min_sleep_between_requests) =
                    min_sleep_between_requests.as_integer()
                {
                    u64::try_from(min_sleep_between_requests).expect(
                        "Min sleep between requests must be between 0 and 18446744073709551615",
                    )
                } else {
                    return Err(
                        ParsePollingErrorKind::MinSleepBetweenRequestsMustBeStringOrInt(
                            min_sleep_between_requests.clone(),
                        ),
                    );
                }
            } else {
                return Err(ParsePollingErrorKind::MinSleepBetweenRequestsNotFound(
                    time.clone(),
                ));
            };

            let max_sleep_between_requests = if let Some(max_sleep_between_requests) =
                time.get("max_sleep_between_requests")
            {
                if let Some(max_sleep_between_requests_str) = max_sleep_between_requests.as_str() {
                    max_sleep_between_requests_str.parse::<u64>().map_err(|_| {
                        ParsePollingErrorKind::MaxSleepBetweenRequestsMustBeStringOrInt(
                            max_sleep_between_requests.clone(),
                        )
                    })?
                } else if let Some(max_sleep_between_requests) =
                    max_sleep_between_requests.as_integer()
                {
                    u64::try_from(max_sleep_between_requests).expect(
                        "Max sleep between requests must be between 0 and 18446744073709551615",
                    )
                } else {
                    return Err(
                        ParsePollingErrorKind::MaxSleepBetweenRequestsMustBeStringOrInt(
                            max_sleep_between_requests.clone(),
                        ),
                    );
                }
            } else {
                return Err(ParsePollingErrorKind::MaxSleepBetweenRequestsNotFound(
                    time.clone(),
                ));
            };

            let request_timeout = if let Some(request_timeout) = time.get("request_timeout") {
                if let Some(request_timeout_str) = request_timeout.as_str() {
                    request_timeout_str.parse::<u64>().map_err(|_| {
                        ParsePollingErrorKind::RequestTimeoutMustBeStringOrInt(
                            request_timeout.clone(),
                        )
                    })?
                } else if let Some(request_timeout) = request_timeout.as_integer() {
                    u64::try_from(request_timeout)
                        .expect("Request timeout must be between 0 and 18446744073709551615")
                } else {
                    return Err(ParsePollingErrorKind::RequestTimeoutMustBeStringOrInt(
                        request_timeout.clone(),
                    ));
                }
            } else {
                return Err(ParsePollingErrorKind::RequestTimeoutNotFound(time.clone()));
            };

            polling_builder = polling_builder.time(time::Time::new(
                min_sleep_between_requests,
                max_sleep_between_requests,
                request_timeout,
            ));
        }
        None => {
            event!(Level::TRACE, "Time not found");
        }
    }

    match polling.get("user_agent") {
        Some(user_agent) => {
            event!(Level::TRACE, "Parse user agent");

            if let Some(value) = user_agent.get("value") {
                if let Some(value) = value.as_str() {
                    polling_builder = polling_builder
                        .user_agent(Some(user_agent::UserAgent::new(value.to_owned())));
                } else {
                    return Err(ParsePollingErrorKind::UserAgentValueMustBeString(
                        value.clone(),
                    ));
                }
            } else {
                event!(Level::TRACE, "User agent value not found");
            }
        }
        None => {
            event!(Level::TRACE, "User agents not found");
        }
    }

    match polling.get("proxy") {
        Some(proxy) => {
            event!(Level::TRACE, "Parse proxy");

            if let Some(value) = proxy.get("value") {
                if let Some(value) = value.as_str() {
                    polling_builder =
                        polling_builder.proxy(Some(proxy::Proxy::new(value.to_owned())));
                } else {
                    return Err(ParsePollingErrorKind::ProxyValueMustBeString(value.clone()));
                }
            } else {
                event!(Level::TRACE, "Proxy value not found");
            }
        }
        None => {
            event!(Level::TRACE, "Proxies not found");
        }
    }

    Ok(polling_builder.build())
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Read file error: {0}")]
    ReadFile(#[from] io::Error),
    #[error(transparent)]
    ParseRoute(#[from] ParseRouteErrorKind),
    #[error(transparent)]
    ParsePolling(#[from] ParsePollingErrorKind),
}

/// Parse rules from toml file
/// # Arguments
/// * `route_path` - Path to route toml file
/// * `polling_path` - Path to polling toml file
/// # Returns
/// Returns [`Rules`] if parsing is successful and all rules are valid, otherwise returns [`ErrorKind`].
/// # Panics
/// - If the port number is not between 0 and 65535
/// - If the max redirects is not between 0 and 65535
/// - If the max depth is not between 0 and 65535
/// - If the min sleep between requests is not between 0 and 18446744073709551615
/// - If the max sleep between requests is not between 0 and 18446744073709551615
/// - If the request timeout is not between 0 and 18446744073709551615
#[instrument(skip_all)]
pub fn parse_rules_from_toml_file(
    route_path: impl AsRef<Path>,
    polling_path: impl AsRef<Path>,
) -> Result<Rules, ErrorKind> {
    let route_path = route_path.as_ref();
    let polling_path = polling_path.as_ref();

    Span::current()
        .record("route_path", field::debug(route_path))
        .record("polling_path", field::debug(polling_path));

    event!(Level::DEBUG, "Parse rules from toml file");

    let route_raw = fs::read_to_string(route_path)?;
    let polling_raw = fs::read_to_string(polling_path)?;

    let rules_builder = Rules::builder()
        .route(parse_route_from_toml(&route_raw)?)
        .polling(parse_polling_from_toml(&polling_raw)?);

    Ok(rules_builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    use glob::Pattern;
    use url::{Host, Url};

    #[test]
    fn test_parse_route_from_toml() {
        let raw = r#"
            [routes]

            [[routes.root_urls]]
            value = "https://example.com"

            [[routes.root_urls]]
            value = "https://example2.com"

            [[routes.hosts.acceptable]]

            [[routes.hosts.acceptable]]
            exact = "example.com"

            [[routes.hosts.acceptable]]
            glob = "example*.com"

            [[routes.hosts.unacceptable]]
            exact = "127.0.0.1"

            [[routes.schemes.acceptable]]
            exact = "https"

            [[routes.schemes.acceptable]]

            [[routes.schemes.acceptable]]
            exact = "http"

            [[routes.schemes.unacceptable]]
            exact = "http"

            [[routes.ports.acceptable]]
            exact = "8080"

            [[routes.ports.acceptable]]
            exact = 80

            [[routes.ports.acceptable]]
            glob = "80*"

            [[routes.ports.unacceptable]]

            [[routes.paths.acceptable]]
            exact = "/example/"

            [[routes.paths.acceptable]]
            glob = "/example2/*"

            [[routes.paths.unacceptable]]
            glob = "/admin/*"

            [[routes.paths.unacceptable]]
        "#;

        let route = parse_route_from_toml(raw).unwrap();

        assert_eq!(route.root_urls.len(), 2);
        assert_eq!(
            **(*route.root_urls).first().unwrap(),
            Url::parse("https://example.com").unwrap(),
        );
        assert_eq!(
            **(*route.root_urls).last().unwrap(),
            Url::parse("https://example2.com").unwrap(),
        );

        assert_eq!(route.hosts.acceptable.len(), 2);
        assert_eq!(
            route.hosts.acceptable[0],
            host::Kind::Exact(Host::Domain("example.com".to_owned()))
        );
        assert_eq!(
            route.hosts.acceptable[1],
            host::Kind::Glob(Pattern::new("example*.com").unwrap())
        );
        assert_eq!(route.hosts.unacceptable.len(), 1);
        assert_eq!(
            route.hosts.unacceptable[0],
            host::Kind::Exact(Host::Ipv4("127.0.0.1".parse().unwrap()))
        );

        assert_eq!(route.schemes.acceptable.len(), 2);
        assert_eq!(route.schemes.acceptable[0], scheme::Kind::Https);
        assert_eq!(route.schemes.acceptable[1], scheme::Kind::Http);
        assert_eq!(route.schemes.unacceptable.len(), 1);
        assert_eq!(route.schemes.unacceptable[0], scheme::Kind::Http);

        assert_eq!(route.ports.acceptable.len(), 3);
        assert_eq!(route.ports.acceptable[0], port::Kind::Exact(8080));
        assert_eq!(route.ports.acceptable[1], port::Kind::Exact(80));
        assert_eq!(
            route.ports.acceptable[2],
            port::Kind::Glob(Pattern::new("80*").unwrap())
        );
        assert_eq!(route.ports.unacceptable.len(), 0);

        assert_eq!(route.paths.acceptable.len(), 2);
        assert_eq!(
            route.paths.acceptable[0],
            path::Kind::Exact("/example/".to_owned())
        );
        assert_eq!(
            route.paths.acceptable[1],
            path::Kind::Glob(Pattern::new("/example2/*").unwrap())
        );
        assert_eq!(route.paths.unacceptable.len(), 1);
        assert_eq!(
            route.paths.unacceptable[0],
            path::Kind::Glob(Pattern::new("/admin/*").unwrap())
        );
    }

    #[test]
    fn test_parse_polling_from_toml() {
        let raw = r#"
            [polling]

            [polling.redirections]
            acceptable = true
            max_redirects = 10

            [polling.depth]
            acceptable = true
            max_depth = 10

            [polling.time]
            min_sleep_between_requests = 1000
            max_sleep_between_requests = 10000
            request_timeout = 1000

            [polling.user_agent]
            value = "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"

            [polling.proxy]
            value = "http://"
        "#;

        let polling = parse_polling_from_toml(raw).unwrap();

        assert_eq!(polling.redirections.acceptable(), true);
        assert_eq!(polling.redirections.max_redirects(), 10);

        assert_eq!(polling.depth.acceptable(), true);
        assert_eq!(polling.depth.max_depth(), 10);

        assert_eq!(polling.time.min_sleep_between_requests, 1000);
        assert_eq!(polling.time.max_sleep_between_requests, 10000);
        assert_eq!(polling.time.request_timeout, 1000);

        assert!(polling.user_agent.is_some());
        assert_eq!(
            *polling.user_agent.unwrap(),
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"
        );

        assert!(polling.proxy.is_some());
        assert_eq!(*polling.proxy.unwrap(), "http://");
    }
}
