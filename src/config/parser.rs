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
use toml::{map::Map, Value};
use tracing::{event, field, instrument, Level, Span};

#[derive(Debug, thiserror::Error)]
pub enum ParseRouteErrorKind {
    #[error("Parse toml error: {0}")]
    ParseToml(#[from] toml::de::Error),

    #[error("Config must be a table, found {0}")]
    ConfigMustBeTable(Value),
    #[error("Routes not found in table: {0}")]
    RoutesNotFound(Map<String, Value>),
    #[error("Routes must be a table, found {0}")]
    RoutesMustBeTable(Value),

    #[error("Root urls must be an array, found {0}")]
    RootUrlsMustBeArray(Value),
    #[error("Root url must be a table, found {0}")]
    RootUrlMustBeTable(Value),
    #[error("Root url value must be a string, found {0}")]
    RootUrlValueMustBeString(Value),
    #[error("Root url parse error: {0}")]
    RootUrlParseError(url::ParseError),

    #[error("Hosts must be a table, found {0}")]
    HostsMustBeTable(Value),
    #[error("Hosts must be an array, found {0}")]
    HostsMustBeArray(Value),
    #[error("Host must be a table, found {0}")]
    HostMustBeTable(Value),
    #[error("Host value must be a string, found {0}")]
    HostExactMustBeString(Value),
    #[error("Host glob must be a string, found {0}")]
    HostGlobMustBeString(Value),
    #[error("Host glob pattern error: {0}")]
    HostGlobPattern(PatternError),
    #[error("Host parse error: {0}")]
    HostParseError(url::ParseError),

    #[error("Schemes must be a table, found {0}")]
    SchemesMustBeTable(Value),
    #[error("Schemes must be an array, found {0}")]
    SchemesMustBeArray(Value),
    #[error("Scheme must be a table, found {0}")]
    SchemeMustBeTable(Value),
    #[error("Scheme value must be a string, found {0}")]
    SchemeExactMustBeString(Value),
    #[error(transparent)]
    UnsupportedScheme(#[from] UnsupportedSchemeError),

    #[error("Ports must be a table, found {0}")]
    PortsMustBeTable(Value),
    #[error("Ports must be an array, found {0}")]
    PortsMustBeArray(Value),
    #[error("Port glob must be a string, found {0}")]
    PortGlobMustBeString(Value),
    #[error("Port glob pattern error: {0}")]
    PortGlobPattern(PatternError),
    #[error("Port value parse error: {0}")]
    PortExactParseError(ParseIntError),
    #[error("Port must be a table, found {0}")]
    PortMustBeTable(Value),
    #[error("Port value must be an int or a string that represents an int, found {0}")]
    PortExactMustBeStringOrInt(Value),

    #[error("Paths must be a table, found {0}")]
    PathsMustBeTable(Value),
    #[error("Paths must be an array, found {0}")]
    PathsMustBeArray(Value),
    #[error("Path glob pattern error: {0}")]
    PathGlobPattern(PatternError),
    #[error("Path glob must be a string, found {0}")]
    PathGlobMustBeString(Value),
    #[error("Path must be a table, found {0}")]
    PathMustBeTable(Value),
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
    let Some(table) = value.as_table() else {
        return Err(ParseRouteErrorKind::ConfigMustBeTable(value));
    };

    let routes = match table.get("routes") {
        Some(routes) => match routes.as_table() {
            Some(routes) => routes,
            None => return Err(ParseRouteErrorKind::RoutesMustBeTable(routes.clone())),
        },
        None => return Err(ParseRouteErrorKind::RoutesNotFound(table.clone())),
    };

    let mut route_builder = Route::builder();

    match routes.get("root_urls") {
        Some(root_urls) => {
            event!(Level::TRACE, "Parse root urls");

            let Some(root_urls) = root_urls.as_array() else {
                return Err(ParseRouteErrorKind::RootUrlsMustBeArray(root_urls.clone()));
            };

            for root_url in root_urls {
                match root_url.as_table() {
                    Some(root_url) => {
                        if let Some(value) = root_url.get("value") {
                            match value.as_str() {
                                Some(root_url) => {
                                    route_builder = route_builder.root_url(
                                        root_url::RootUrl::new(root_url)
                                            .map_err(ParseRouteErrorKind::RootUrlParseError)?,
                                    );

                                    continue;
                                }
                                None => {
                                    return Err(ParseRouteErrorKind::RootUrlValueMustBeString(
                                        value.clone(),
                                    ))
                                }
                            }
                        }

                        event!(Level::TRACE, "Root url exact not found");
                    }
                    None => return Err(ParseRouteErrorKind::RootUrlMustBeTable(root_url.clone())),
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

            let Some(hosts) = hosts.as_table() else {
                return Err(ParseRouteErrorKind::HostsMustBeTable(hosts.clone()));
            };

            match hosts.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable hosts");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ParseRouteErrorKind::HostsMustBeArray(acceptable.clone()));
                    };

                    for host in acceptable {
                        match host.as_table() {
                            Some(host) => {
                                if let Some(glob) = host.get("glob") {
                                    match glob.as_str() {
                                        Some(host) => {
                                            route_builder = route_builder.host(host::Matcher::new(
                                                PermissionKind::Acceptable,
                                                host::Kind::glob(host).map_err(
                                                    ParseRouteErrorKind::HostGlobPattern,
                                                )?,
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
                            None => return Err(ParseRouteErrorKind::HostMustBeTable(host.clone())),
                        }
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
                        match host.as_table() {
                            Some(host) => {
                                if let Some(glob) = host.get("glob") {
                                    match glob.as_str() {
                                        Some(host) => {
                                            route_builder = route_builder.host(host::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                host::Kind::glob(host).map_err(
                                                    ParseRouteErrorKind::HostGlobPattern,
                                                )?,
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
                            None => return Err(ParseRouteErrorKind::HostMustBeTable(host.clone())),
                        }
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

            let Some(schemes) = schemes.as_table() else {
                return Err(ParseRouteErrorKind::SchemesMustBeTable(schemes.clone()));
            };

            match schemes.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable schemes");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ParseRouteErrorKind::SchemesMustBeArray(acceptable.clone()));
                    };

                    for scheme in acceptable {
                        match scheme.as_table() {
                            Some(scheme) => {
                                if let Some(exact) = scheme.get("exact") {
                                    match exact.as_str() {
                                        Some(scheme) => {
                                            route_builder =
                                                route_builder.scheme(scheme::Matcher::new(
                                                    PermissionKind::Acceptable,
                                                    scheme::Kind::try_from(scheme.to_owned())?,
                                                ));

                                            continue;
                                        }
                                        None => {
                                            return Err(
                                                ParseRouteErrorKind::SchemeExactMustBeString(
                                                    exact.clone(),
                                                ),
                                            )
                                        }
                                    }
                                }

                                event!(Level::TRACE, "Scheme exact not found");
                            }
                            None => {
                                return Err(ParseRouteErrorKind::SchemeMustBeTable(scheme.clone()))
                            }
                        }
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
                        match scheme.as_table() {
                            Some(scheme) => {
                                if let Some(exact) = scheme.get("exact") {
                                    match exact.as_str() {
                                        Some(scheme) => {
                                            route_builder =
                                                route_builder.scheme(scheme::Matcher::new(
                                                    PermissionKind::Unacceptable,
                                                    scheme::Kind::try_from(scheme.to_owned())?,
                                                ));

                                            continue;
                                        }
                                        None => {
                                            return Err(
                                                ParseRouteErrorKind::SchemeExactMustBeString(
                                                    exact.clone(),
                                                ),
                                            )
                                        }
                                    }
                                }

                                event!(Level::TRACE, "Scheme exact not found");
                            }
                            None => {
                                return Err(ParseRouteErrorKind::SchemeMustBeTable(scheme.clone()))
                            }
                        }
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

            let Some(ports) = ports.as_table() else {
                return Err(ParseRouteErrorKind::PortsMustBeTable(ports.clone()));
            };

            match ports.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable ports");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ParseRouteErrorKind::PortsMustBeArray(acceptable.clone()));
                    };

                    for port in acceptable {
                        match port.as_table() {
                            Some(port) => {
                                if let Some(glob) = port.get("glob") {
                                    match glob.as_str() {
                                        Some(port) => {
                                            route_builder = route_builder.port(port::Matcher::new(
                                                PermissionKind::Acceptable,
                                                port::Kind::glob(port).map_err(
                                                    ParseRouteErrorKind::PortGlobPattern,
                                                )?,
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
                                            port::Kind::exact_str(port).map_err(
                                                ParseRouteErrorKind::PortExactParseError,
                                            )?,
                                        ));

                                        continue;
                                    }

                                    if let Some(port) = exact.as_integer() {
                                        route_builder =
                                            route_builder.port(port::Matcher::new(
                                                PermissionKind::Acceptable,
                                                port::Kind::exact(u16::try_from(port).expect(
                                                    "Port number must be between 0 and 65535",
                                                )),
                                            ));

                                        continue;
                                    }

                                    return Err(ParseRouteErrorKind::PortExactMustBeStringOrInt(
                                        exact.clone(),
                                    ));
                                }

                                event!(Level::TRACE, "Port exact not found");
                            }
                            None => return Err(ParseRouteErrorKind::PortMustBeTable(port.clone())),
                        }
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
                        match port.as_table() {
                            Some(port) => {
                                if let Some(glob) = port.get("glob") {
                                    match glob.as_str() {
                                        Some(port) => {
                                            route_builder = route_builder.port(port::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                port::Kind::glob(port).map_err(
                                                    ParseRouteErrorKind::PortGlobPattern,
                                                )?,
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
                                            port::Kind::exact_str(port).map_err(
                                                ParseRouteErrorKind::PortExactParseError,
                                            )?,
                                        ));

                                        continue;
                                    }

                                    if let Some(port) = exact.as_integer() {
                                        route_builder =
                                            route_builder.port(port::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                port::Kind::exact(u16::try_from(port).expect(
                                                    "Port number must be between 0 and 65535",
                                                )),
                                            ));

                                        continue;
                                    }

                                    return Err(ParseRouteErrorKind::PortExactMustBeStringOrInt(
                                        exact.clone(),
                                    ));
                                }

                                event!(Level::TRACE, "Port exact not found");
                            }
                            None => return Err(ParseRouteErrorKind::PortMustBeTable(port.clone())),
                        }
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

            let Some(paths) = paths.as_table() else {
                return Err(ParseRouteErrorKind::PathsMustBeTable(paths.clone()));
            };

            match paths.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable paths");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ParseRouteErrorKind::PathsMustBeArray(acceptable.clone()));
                    };

                    for path in acceptable {
                        match path.as_table() {
                            Some(path) => {
                                if let Some(glob) = path.get("glob") {
                                    match glob.as_str() {
                                        Some(path) => {
                                            route_builder = route_builder.path(path::Matcher::new(
                                                PermissionKind::Acceptable,
                                                path::Kind::glob(path).map_err(
                                                    ParseRouteErrorKind::PathGlobPattern,
                                                )?,
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
                            None => return Err(ParseRouteErrorKind::PathMustBeTable(path.clone())),
                        }
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
                        match path.as_table() {
                            Some(path) => {
                                if let Some(glob) = path.get("glob") {
                                    match glob.as_str() {
                                        Some(path) => {
                                            route_builder = route_builder.path(path::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                path::Kind::glob(path).map_err(
                                                    ParseRouteErrorKind::PathGlobPattern,
                                                )?,
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
                            None => return Err(ParseRouteErrorKind::PathMustBeTable(path.clone())),
                        }
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

    #[error("Config must be a table, found {0}")]
    ConfigMustBeTable(Value),
    #[error("Polling not found in table: {0}")]
    PollingNotFound(Map<String, Value>),
    #[error("Polling must be a table, found {0}")]
    PollingMustBeTable(Value),

    #[error("Redirections must be a table, found {0}")]
    RedirectionsMustBeTable(Value),
    #[error("Redirections acceptable not found in table: {0}")]
    RedirectionsAcceptableNotFound(Map<String, Value>),
    #[error("Redirections acceptable must be a bool, found {0}")]
    RedirectionsAcceptableMustBeBool(Value),
    #[error("Redirections max redirects not found in table: {0}")]
    RedirectionsMaxRedirectsNotFound(Map<String, Value>),
    #[error(
        "Redirections max redirects must be an int or a string that represents an int, found {0}"
    )]
    RedirectionsMaxRedirectsMustBeStringOrInt(Value),

    #[error("Depth must be a table, found {0}")]
    DepthMustBeTable(Value),
    #[error("Depth acceptable not found in table: {0}")]
    DepthAcceptableNotFound(Map<String, Value>),
    #[error("Depth acceptable must be a bool, found {0}")]
    DepthAcceptableMustBeBool(Value),
    #[error("Depth max redirects not found in table: {0}")]
    DepthMaxRedirectsNotFound(Map<String, Value>),
    #[error("Depth max depth must be an int or a string that represents an int, found {0}")]
    DepthMaxDepthMustBeStringOrInt(Value),

    #[error("Time must be a table, found {0}")]
    TimeMustBeTable(Value),
    #[error("Min sleep between requests not found in table: {0}")]
    MinSleepBetweenRequestsNotFound(Map<String, Value>),
    #[error("Min sleep between requests value must be an int or a string that represents an int, found {0}")]
    MinSleepBetweenRequestsMustBeStringOrInt(Value),
    #[error("Max sleep between requests not found in table: {0}")]
    MaxSleepBetweenRequestsNotFound(Map<String, Value>),
    #[error("Max sleep between requests value must be an int or a string that represents an int, found {0}")]
    MaxSleepBetweenRequestsMustBeStringOrInt(Value),
    #[error("Request timeout not found in table: {0}")]
    RequestTimeoutNotFound(Map<String, Value>),
    #[error("Request timeout value must be an int or a string that represents an int, found {0}")]
    RequestTimeoutMustBeStringOrInt(Value),

    #[error("User agents must be an array, found {0}")]
    UserAgentsMustBeArray(Value),
    #[error("User agent must be a table, found {0}")]
    UserAgentMustBeTable(Value),
    #[error("User agent value must be a string, found {0}")]
    UserAgentValueMustBeString(Value),

    #[error("Proxies must be an array, found {0}")]
    ProxiesMustBeArray(Value),
    #[error("Proxy must be a table, found {0}")]
    ProxyMustBeTable(Value),
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
    let Some(table) = value.as_table() else {
        return Err(ParsePollingErrorKind::ConfigMustBeTable(value));
    };

    let polling = match table.get("polling") {
        Some(polling) => match polling.as_table() {
            Some(polling) => polling,
            None => return Err(ParsePollingErrorKind::PollingMustBeTable(polling.clone())),
        },
        None => return Err(ParsePollingErrorKind::PollingNotFound(table.clone())),
    };

    let mut polling_builder = Polling::builder();

    match polling.get("redirections") {
        Some(redirections) => {
            event!(Level::TRACE, "Parse redirections");

            let Some(redirections) = redirections.as_table() else {
                return Err(ParsePollingErrorKind::RedirectionsMustBeTable(
                    redirections.clone(),
                ));
            };

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

            let Some(depth) = depth.as_table() else {
                return Err(ParsePollingErrorKind::DepthMustBeTable(depth.clone()));
            };

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

            let Some(time) = time.as_table() else {
                return Err(ParsePollingErrorKind::TimeMustBeTable(time.clone()));
            };

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

    match polling.get("user_agents") {
        Some(user_agents) => {
            event!(Level::TRACE, "Parse user agents");

            let Some(user_agents) = user_agents.as_array() else {
                return Err(ParsePollingErrorKind::UserAgentsMustBeArray(
                    user_agents.clone(),
                ));
            };

            for user_agent in user_agents {
                match user_agent.as_table() {
                    Some(user_agent) => {
                        if let Some(value) = user_agent.get("value") {
                            if let Some(value) = value.as_str() {
                                polling_builder = polling_builder
                                    .user_agent(user_agent::UserAgent::new(value.to_owned()));

                                continue;
                            }

                            return Err(ParsePollingErrorKind::UserAgentValueMustBeString(
                                value.clone(),
                            ));
                        }

                        event!(Level::TRACE, "User agent value not found");
                    }
                    None => {
                        return Err(ParsePollingErrorKind::UserAgentMustBeTable(
                            user_agent.clone(),
                        ))
                    }
                }
            }
        }
        None => {
            event!(Level::TRACE, "User agents not found");
        }
    }

    match polling.get("proxies") {
        Some(proxies) => {
            event!(Level::TRACE, "Parse proxies");

            let Some(proxies) = proxies.as_array() else {
                return Err(ParsePollingErrorKind::ProxiesMustBeArray(proxies.clone()));
            };

            for proxy in proxies {
                match proxy.as_table() {
                    Some(proxy) => {
                        if let Some(value) = proxy.get("value") {
                            if let Some(value) = value.as_str() {
                                polling_builder =
                                    polling_builder.proxy(proxy::Proxy::new(value.to_owned()));

                                continue;
                            }

                            return Err(ParsePollingErrorKind::ProxyValueMustBeString(
                                value.clone(),
                            ));
                        }

                        event!(Level::TRACE, "Proxy value not found");
                    }
                    None => return Err(ParsePollingErrorKind::ProxyMustBeTable(proxy.clone())),
                }
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

            [[polling.user_agents]]
            value = "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"

            [[polling.user_agents]]

            [[polling.proxies]]
            value = "http://"

            [[polling.proxies]]
        "#;

        let polling = parse_polling_from_toml(raw).unwrap();

        assert_eq!(polling.redirections.acceptable(), true);
        assert_eq!(polling.redirections.max_redirects(), 10);

        assert_eq!(polling.depth.acceptable(), true);
        assert_eq!(polling.depth.max_depth(), 10);

        assert_eq!(polling.time.min_sleep_between_requests, 1000);
        assert_eq!(polling.time.max_sleep_between_requests, 10000);
        assert_eq!(polling.time.request_timeout, 1000);

        assert_eq!(polling.user_agents.len(), 1);
        assert_eq!(
            **(*polling.user_agents).first().unwrap(),
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"
        );

        assert_eq!(polling.proxies.len(), 1);
        assert_eq!(**(*polling.proxies).first().unwrap(), "http://");
    }
}
