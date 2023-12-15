use crate::models::{
    route::Route,
    routes::{
        host,
        method::{self, UnsupportedMethodError},
        path,
        permission::Kind as PermissionKind,
        port,
        scheme::{self, UnsupportedSchemeError},
    },
    rules::Rules,
};

use glob::PatternError;
use std::num::ParseIntError;
use toml::{map::Map, Value};
use tracing::{event, instrument, Level};

/// Error kind for [`parse_rules_from_toml`].
/// # Notes
/// This error kind is used to indicate which rule is invalid and why it is invalid.
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Parse toml error: {0}")]
    ParseToml(#[from] toml::de::Error),

    #[error("Config must be a table, found {0}")]
    ConfigMustBeTable(Value),
    #[error("Routes not found in table: {0}")]
    RoutesNotFound(Map<String, Value>),
    #[error("Routes must be a table, found {0}")]
    RoutesMustBeTable(Value),

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
    HostParseError(#[from] url::ParseError),

    #[error("Methods must be a table, found {0}")]
    MethodsMustBeTable(Value),
    #[error("Methods must be an array, found {0}")]
    MethodsMustBeArray(Value),
    #[error("Method must be a table, found {0}")]
    MethodMustBeTable(Value),
    #[error("Method value must be a string, found {0}")]
    MethodExactMustBeString(Value),
    #[error(transparent)]
    UnsupportedMethod(#[from] UnsupportedMethodError),

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

/// Parse rules from toml
/// # Arguments
/// * `raw` - Raw toml string
/// # Returns
/// Returns [`Rules`] if parsing is successful and all rules are valid, otherwise returns [`ErrorKind`].
/// # Panics
/// If the port number is not between 0 and 65535
#[instrument(skip_all)]
pub fn parse_rules_from_toml(raw: &str) -> Result<Rules, ErrorKind> {
    event!(Level::DEBUG, "Parse rules from toml");

    let value = raw.parse::<Value>()?;
    let Some(table) = value.as_table() else {
        return Err(ErrorKind::ConfigMustBeTable(value));
    };

    let routes = match table.get("routes") {
        Some(routes) => match routes.as_table() {
            Some(routes) => routes,
            None => return Err(ErrorKind::RoutesMustBeTable(routes.clone())),
        },
        None => return Err(ErrorKind::RoutesNotFound(table.clone())),
    };

    let mut rules_builder = Rules::builder();

    let mut route_builder = Route::builder();

    match routes.get("hosts") {
        Some(hosts) => {
            event!(Level::TRACE, "Parse hosts");

            let Some(hosts) = hosts.as_table() else {
                return Err(ErrorKind::HostsMustBeTable(hosts.clone()));
            };

            match hosts.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable hosts");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ErrorKind::HostsMustBeArray(acceptable.clone()));
                    };

                    for host in acceptable {
                        match host.as_table() {
                            Some(host) => {
                                if let Some(glob) = host.get("glob") {
                                    match glob.as_str() {
                                        Some(host) => {
                                            route_builder = route_builder.host(host::Matcher::new(
                                                PermissionKind::Acceptable,
                                                host::Kind::glob(host)
                                                    .map_err(ErrorKind::HostGlobPattern)?,
                                            ));

                                            continue;
                                        }
                                        None => {
                                            return Err(ErrorKind::HostGlobMustBeString(
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
                                                host::Kind::exact(host)?,
                                            ));
                                        }
                                        None => {
                                            return Err(ErrorKind::HostExactMustBeString(
                                                exact.clone(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Host exact not found");
                                }
                            }
                            None => return Err(ErrorKind::HostMustBeTable(host.clone())),
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
                        return Err(ErrorKind::HostsMustBeArray(unacceptable.clone()));
                    };

                    for host in unacceptable {
                        match host.as_table() {
                            Some(host) => {
                                if let Some(glob) = host.get("glob") {
                                    match glob.as_str() {
                                        Some(host) => {
                                            route_builder = route_builder.host(host::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                host::Kind::glob(host)
                                                    .map_err(ErrorKind::HostGlobPattern)?,
                                            ));

                                            continue;
                                        }
                                        None => {
                                            return Err(ErrorKind::HostGlobMustBeString(
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
                                                host::Kind::exact(host)?,
                                            ));
                                        }
                                        None => {
                                            return Err(ErrorKind::HostExactMustBeString(
                                                exact.clone(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Host exact not found");
                                }
                            }
                            None => return Err(ErrorKind::HostMustBeTable(host.clone())),
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

    match routes.get("methods") {
        Some(methods) => {
            event!(Level::TRACE, "Parse methods");

            let Some(methods) = methods.as_table() else {
                return Err(ErrorKind::MethodsMustBeTable(methods.clone()));
            };

            match methods.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable methods");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ErrorKind::MethodsMustBeArray(acceptable.clone()));
                    };

                    for method in acceptable {
                        match method.as_table() {
                            Some(method) => {
                                if let Some(exact) = method.get("exact") {
                                    match exact.as_str() {
                                        Some(method) => {
                                            route_builder =
                                                route_builder.method(method::Matcher::new(
                                                    PermissionKind::Acceptable,
                                                    method::Kind::try_from(method.to_owned())?,
                                                ));
                                        }
                                        None => {
                                            return Err(ErrorKind::MethodExactMustBeString(
                                                exact.clone(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Method exact not found");
                                }
                            }
                            None => return Err(ErrorKind::MethodMustBeTable(method.clone())),
                        }
                    }
                }
                None => {
                    event!(Level::TRACE, "Acceptable methods not found");
                }
            }

            match methods.get("unacceptable") {
                Some(unacceptable) => {
                    event!(Level::TRACE, "Parse unacceptable methods");

                    let Some(unacceptable) = unacceptable.as_array() else {
                        return Err(ErrorKind::MethodsMustBeArray(unacceptable.clone()));
                    };

                    for method in unacceptable {
                        match method.as_table() {
                            Some(method) => {
                                if let Some(exact) = method.get("exact") {
                                    match exact.as_str() {
                                        Some(method) => {
                                            route_builder =
                                                route_builder.method(method::Matcher::new(
                                                    PermissionKind::Unacceptable,
                                                    method::Kind::try_from(method.to_owned())?,
                                                ));
                                        }
                                        None => {
                                            return Err(ErrorKind::MethodExactMustBeString(
                                                exact.clone(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Method exact not found");
                                }
                            }
                            None => return Err(ErrorKind::MethodMustBeTable(method.clone())),
                        }
                    }
                }
                None => {
                    event!(Level::TRACE, "Unacceptable methods not found");
                }
            }
        }
        None => {
            event!(Level::TRACE, "Methods not found");
        }
    }

    match routes.get("schemes") {
        Some(schemes) => {
            event!(Level::TRACE, "Parse schemes");

            let Some(schemes) = schemes.as_table() else {
                return Err(ErrorKind::SchemesMustBeTable(schemes.clone()));
            };

            match schemes.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable schemes");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ErrorKind::SchemesMustBeArray(acceptable.clone()));
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
                                        }
                                        None => {
                                            return Err(ErrorKind::SchemeExactMustBeString(
                                                exact.clone(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Scheme exact not found");
                                }
                            }
                            None => return Err(ErrorKind::SchemeMustBeTable(scheme.clone())),
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
                        return Err(ErrorKind::SchemesMustBeArray(unacceptable.clone()));
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
                                        }
                                        None => {
                                            return Err(ErrorKind::SchemeExactMustBeString(
                                                exact.clone(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Scheme exact not found");
                                }
                            }
                            None => return Err(ErrorKind::SchemeMustBeTable(scheme.clone())),
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
                return Err(ErrorKind::PortsMustBeTable(ports.clone()));
            };

            match ports.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable ports");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ErrorKind::PortsMustBeArray(acceptable.clone()));
                    };

                    for port in acceptable {
                        match port.as_table() {
                            Some(port) => {
                                if let Some(glob) = port.get("glob") {
                                    match glob.as_str() {
                                        Some(port) => {
                                            route_builder = route_builder.port(port::Matcher::new(
                                                PermissionKind::Acceptable,
                                                port::Kind::glob(port)
                                                    .map_err(ErrorKind::PortGlobPattern)?,
                                            ));

                                            continue;
                                        }
                                        None => {
                                            return Err(ErrorKind::PortGlobMustBeString(
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
                                                .map_err(ErrorKind::PortExactParseError)?,
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

                                    return Err(ErrorKind::PortExactMustBeStringOrInt(
                                        exact.clone(),
                                    ));
                                }

                                event!(Level::TRACE, "Port exact not found");
                            }
                            None => return Err(ErrorKind::PortMustBeTable(port.clone())),
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
                        return Err(ErrorKind::PortsMustBeArray(unacceptable.clone()));
                    };

                    for port in unacceptable {
                        match port.as_table() {
                            Some(port) => {
                                if let Some(glob) = port.get("glob") {
                                    match glob.as_str() {
                                        Some(port) => {
                                            route_builder = route_builder.port(port::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                port::Kind::glob(port)
                                                    .map_err(ErrorKind::PortGlobPattern)?,
                                            ));

                                            continue;
                                        }
                                        None => {
                                            return Err(ErrorKind::PortGlobMustBeString(
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
                                                .map_err(ErrorKind::PortExactParseError)?,
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

                                    return Err(ErrorKind::PortExactMustBeStringOrInt(
                                        exact.clone(),
                                    ));
                                }

                                event!(Level::TRACE, "Port exact not found");
                            }
                            None => return Err(ErrorKind::PortMustBeTable(port.clone())),
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
                return Err(ErrorKind::PathsMustBeTable(paths.clone()));
            };

            match paths.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable paths");

                    let Some(acceptable) = acceptable.as_array() else {
                        return Err(ErrorKind::PathsMustBeArray(acceptable.clone()));
                    };

                    for path in acceptable {
                        match path.as_table() {
                            Some(path) => {
                                if let Some(glob) = path.get("glob") {
                                    match glob.as_str() {
                                        Some(path) => {
                                            route_builder = route_builder.path(path::Matcher::new(
                                                PermissionKind::Acceptable,
                                                path::Kind::glob(path)
                                                    .map_err(ErrorKind::PathGlobPattern)?,
                                            ));

                                            continue;
                                        }
                                        None => {
                                            return Err(ErrorKind::PathGlobMustBeString(
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
                                        }
                                        None => {
                                            return Err(ErrorKind::PathExactMustBeString(
                                                exact.clone(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Path exact not found");
                                }
                            }
                            None => return Err(ErrorKind::PathMustBeTable(path.clone())),
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
                        return Err(ErrorKind::PathsMustBeArray(unacceptable.clone()));
                    };

                    for path in unacceptable {
                        match path.as_table() {
                            Some(path) => {
                                if let Some(glob) = path.get("glob") {
                                    match glob.as_str() {
                                        Some(path) => {
                                            route_builder = route_builder.path(path::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                path::Kind::glob(path)
                                                    .map_err(ErrorKind::PathGlobPattern)?,
                                            ));

                                            continue;
                                        }
                                        None => {
                                            return Err(ErrorKind::PathGlobMustBeString(
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
                                        }
                                        None => {
                                            return Err(ErrorKind::PathExactMustBeString(
                                                exact.clone(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Path exact not found");
                                }
                            }
                            None => return Err(ErrorKind::PathMustBeTable(path.clone())),
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

    rules_builder = rules_builder.route(route_builder.build());

    Ok(rules_builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    use glob::Pattern;
    use url::Host;

    #[test]
    fn test_parse_rules_from_toml() {
        let raw = r#"
            title = "Route rules"

            [routes]

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

            [[routes.methods.acceptable]]
            exact = "GET"

            [[routes.methods.acceptable]]
            exact = "PATCH"

            [[routes.methods.unacceptable]]
            exact = "POST"
        "#;

        let rules = parse_rules_from_toml(raw).unwrap();
        let route = rules.route;

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

        assert_eq!(route.methods.acceptable.len(), 2);
        assert_eq!(route.methods.acceptable[0], method::Kind::Get);
        assert_eq!(route.methods.acceptable[1], method::Kind::Patch);
        assert_eq!(route.methods.unacceptable.len(), 1);
        assert_eq!(route.methods.unacceptable[0], method::Kind::Post);
    }
}
