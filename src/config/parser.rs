use crate::models::{
    route::Route,
    routes::{host, method, path, permission::Kind as PermissionKind, port, scheme},
    rules::Rules,
};

use tracing::{event, instrument, Level};

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Parse toml error: {0}")]
    ParseToml(#[from] toml::de::Error),

    #[error("Config must be a table, found {0}")]
    ConfigMustBeTable(toml::Value),
    #[error("Routes not found in table: {0}")]
    RoutesNotFound(toml::map::Map<String, toml::Value>),
    #[error("Routes must be a table, found {0}")]
    RoutesMustBeTable(toml::Value),

    #[error("Hosts must be a table, found {0}")]
    HostsMustBeTable(toml::Value),
    #[error("Hosts must be an array, found {0}")]
    HostsMustBeArray(toml::Value),
    #[error("Host must be a table, found {0}")]
    HostMustBeTable(toml::Value),
    #[error("Host value must be a string, found {0}")]
    HostValueMustBeString(toml::Value),
    #[error(transparent)]
    Host(#[from] host::ErrorKind),

    #[error("Methods must be a table, found {0}")]
    MethodsMustBeTable(toml::Value),
    #[error("Methods must be an array, found {0}")]
    MethodsMustBeArray(toml::Value),
    #[error("Method must be a table, found {0}")]
    MethodMustBeTable(toml::Value),
    #[error("Method value must be a string, found {0}")]
    MethodValueMustBeString(toml::Value),
    #[error(transparent)]
    UnsupportedMethod(#[from] method::UnsupportedMethodError),

    #[error("Schemes must be a table, found {0}")]
    SchemesMustBeTable(toml::Value),
    #[error("Schemes must be an array, found {0}")]
    SchemesMustBeArray(toml::Value),
    #[error("Scheme must be a table, found {0}")]
    SchemeMustBeTable(toml::Value),
    #[error("Scheme value must be a string, found {0}")]
    SchemeValueMustBeString(toml::Value),
    #[error(transparent)]
    UnsupportedScheme(#[from] scheme::UnsupportedSchemeError),

    #[error("Ports must be a table, found {0}")]
    PortsMustBeTable(toml::Value),
    #[error("Ports must be an array, found {0}")]
    PortsMustBeArray(toml::Value),
    #[error("Port must be a table, found {0}")]
    PortMustBeTable(toml::Value),
    #[error("Port value must be a string or an int, found {0}")]
    PortValueMustBeStringOrInt(toml::Value),
    #[error(transparent)]
    Port(#[from] port::ErrorKind),

    #[error("Paths must be a table, found {0}")]
    PathsMustBeTable(toml::Value),
    #[error("Paths must be an array, found {0}")]
    PathsMustBeArray(toml::Value),
    #[error("Path must be a table, found {0}")]
    PathMustBeTable(toml::Value),
    #[error("Path value must be a string, found {0}")]
    PathValueMustBeString(toml::Value),
    #[error(transparent)]
    Path(#[from] path::ErrorKind),
}

#[instrument(skip_all)]
pub fn parse_rules_from_toml(raw: &str) -> Result<Rules, ErrorKind> {
    event!(Level::DEBUG, "Parse rules from toml");

    let value = raw.parse::<toml::Value>()?;
    let table = match value.as_table() {
        Some(table) => table,
        None => return Err(ErrorKind::ConfigMustBeTable(value)),
    };

    let routes = match table.get("routes") {
        Some(routes) => match routes.as_table() {
            Some(routes) => routes,
            None => return Err(ErrorKind::RoutesMustBeTable(routes.to_owned())),
        },
        None => return Err(ErrorKind::RoutesNotFound(table.to_owned())),
    };

    let mut rules_builder = Rules::builder();

    let mut route_builder = Route::builder();

    match routes.get("hosts") {
        Some(hosts) => {
            event!(Level::TRACE, "Parse hosts");

            let hosts = match hosts.as_table() {
                Some(hosts) => hosts,
                None => return Err(ErrorKind::HostsMustBeTable(hosts.to_owned())),
            };

            match hosts.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable hosts");

                    let acceptable = match acceptable.as_array() {
                        Some(acceptable) => acceptable,
                        None => return Err(ErrorKind::HostsMustBeArray(acceptable.to_owned())),
                    };

                    for host in acceptable {
                        match host.as_table() {
                            Some(host) => {
                                if let Some(value) = host.get("value") {
                                    match value.as_str() {
                                        Some(host) => {
                                            route_builder = route_builder.host(host::Matcher::new(
                                                PermissionKind::Acceptable,
                                                host::Kind::try_from(host.to_owned())?,
                                            ));
                                        }
                                        None => {
                                            return Err(ErrorKind::HostValueMustBeString(
                                                value.to_owned(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Host value not found");
                                }
                            }
                            None => return Err(ErrorKind::HostMustBeTable(host.to_owned())),
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

                    let unacceptable = match unacceptable.as_array() {
                        Some(unacceptable) => unacceptable,
                        None => return Err(ErrorKind::HostsMustBeArray(unacceptable.to_owned())),
                    };

                    for host in unacceptable {
                        match host.as_table() {
                            Some(host) => {
                                if let Some(value) = host.get("value") {
                                    match value.as_str() {
                                        Some(host) => {
                                            route_builder = route_builder.host(host::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                host::Kind::try_from(host.to_owned())?,
                                            ));
                                        }
                                        None => {
                                            return Err(ErrorKind::HostValueMustBeString(
                                                value.to_owned(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Host value not found");
                                }
                            }
                            None => return Err(ErrorKind::HostMustBeTable(host.to_owned())),
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

            let methods = match methods.as_table() {
                Some(methods) => methods,
                None => return Err(ErrorKind::MethodsMustBeTable(methods.to_owned())),
            };

            match methods.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable methods");

                    let acceptable = match acceptable.as_array() {
                        Some(acceptable) => acceptable,
                        None => return Err(ErrorKind::MethodsMustBeArray(acceptable.to_owned())),
                    };

                    for method in acceptable {
                        match method.as_table() {
                            Some(method) => {
                                if let Some(value) = method.get("value") {
                                    match value.as_str() {
                                        Some(method) => {
                                            route_builder =
                                                route_builder.method(method::Matcher::new(
                                                    PermissionKind::Acceptable,
                                                    method::Kind::try_from(method.to_owned())?,
                                                ));
                                        }
                                        None => {
                                            return Err(ErrorKind::MethodValueMustBeString(
                                                value.to_owned(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Method value not found");
                                }
                            }
                            None => return Err(ErrorKind::MethodMustBeTable(method.to_owned())),
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

                    let unacceptable = match unacceptable.as_array() {
                        Some(unacceptable) => unacceptable,
                        None => return Err(ErrorKind::MethodsMustBeArray(unacceptable.to_owned())),
                    };

                    for method in unacceptable {
                        match method.as_table() {
                            Some(method) => {
                                if let Some(value) = method.get("value") {
                                    match value.as_str() {
                                        Some(method) => {
                                            route_builder =
                                                route_builder.method(method::Matcher::new(
                                                    PermissionKind::Unacceptable,
                                                    method::Kind::try_from(method.to_owned())?,
                                                ));
                                        }
                                        None => {
                                            return Err(ErrorKind::MethodValueMustBeString(
                                                value.to_owned(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Method value not found");
                                }
                            }
                            None => return Err(ErrorKind::MethodMustBeTable(method.to_owned())),
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

            let schemes = match schemes.as_table() {
                Some(schemes) => schemes,
                None => return Err(ErrorKind::SchemesMustBeTable(schemes.to_owned())),
            };

            match schemes.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable schemes");

                    let acceptable = match acceptable.as_array() {
                        Some(acceptable) => acceptable,
                        None => return Err(ErrorKind::SchemesMustBeArray(acceptable.to_owned())),
                    };

                    for scheme in acceptable {
                        match scheme.as_table() {
                            Some(scheme) => {
                                if let Some(value) = scheme.get("value") {
                                    match value.as_str() {
                                        Some(scheme) => {
                                            route_builder =
                                                route_builder.scheme(scheme::Matcher::new(
                                                    PermissionKind::Acceptable,
                                                    scheme::Kind::try_from(scheme.to_owned())?,
                                                ));
                                        }
                                        None => {
                                            return Err(ErrorKind::SchemeValueMustBeString(
                                                value.to_owned(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Scheme value not found");
                                }
                            }
                            None => return Err(ErrorKind::SchemeMustBeTable(scheme.to_owned())),
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

                    let unacceptable = match unacceptable.as_array() {
                        Some(unacceptable) => unacceptable,
                        None => return Err(ErrorKind::SchemesMustBeArray(unacceptable.to_owned())),
                    };

                    for scheme in unacceptable {
                        match scheme.as_table() {
                            Some(scheme) => {
                                if let Some(value) = scheme.get("value") {
                                    match value.as_str() {
                                        Some(scheme) => {
                                            route_builder =
                                                route_builder.scheme(scheme::Matcher::new(
                                                    PermissionKind::Unacceptable,
                                                    scheme::Kind::try_from(scheme.to_owned())?,
                                                ));
                                        }
                                        None => {
                                            return Err(ErrorKind::SchemeValueMustBeString(
                                                value.to_owned(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Scheme value not found");
                                }
                            }
                            None => return Err(ErrorKind::SchemeMustBeTable(scheme.to_owned())),
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

            let ports = match ports.as_table() {
                Some(ports) => ports,
                None => return Err(ErrorKind::PortsMustBeTable(ports.to_owned())),
            };

            match ports.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable ports");

                    let acceptable = match acceptable.as_array() {
                        Some(acceptable) => acceptable,
                        None => return Err(ErrorKind::PortsMustBeArray(acceptable.to_owned())),
                    };

                    for port in acceptable {
                        match port.as_table() {
                            Some(port) => {
                                if let Some(value) = port.get("value") {
                                    match value.as_str() {
                                        Some(port) => {
                                            route_builder = route_builder.port(port::Matcher::new(
                                                PermissionKind::Acceptable,
                                                port::Kind::try_from(port.to_owned())?,
                                            ));

                                            continue;
                                        }
                                        None => {}
                                    }

                                    match value.as_integer() {
                                        Some(port) => {
                                            route_builder = route_builder.port(port::Matcher::new(
                                                PermissionKind::Acceptable,
                                                port::Kind::from(port as u16),
                                            ));

                                            continue;
                                        }
                                        None => {}
                                    }

                                    return Err(ErrorKind::PortValueMustBeStringOrInt(
                                        value.to_owned(),
                                    ));
                                } else {
                                    event!(Level::TRACE, "Port value not found");

                                    route_builder = route_builder.port(port::Matcher::new(
                                        PermissionKind::Acceptable,
                                        port::Kind::Any,
                                    ));
                                }
                            }
                            None => return Err(ErrorKind::PortMustBeTable(port.to_owned())),
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

                    let unacceptable = match unacceptable.as_array() {
                        Some(unacceptable) => unacceptable,
                        None => return Err(ErrorKind::PortsMustBeArray(unacceptable.to_owned())),
                    };

                    for port in unacceptable {
                        match port.as_table() {
                            Some(port) => {
                                if let Some(value) = port.get("value") {
                                    match value.as_str() {
                                        Some(port) => {
                                            route_builder = route_builder.port(port::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                port::Kind::try_from(port.to_owned())?,
                                            ));

                                            continue;
                                        }
                                        None => {}
                                    }

                                    match value.as_integer() {
                                        Some(port) => {
                                            route_builder = route_builder.port(port::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                port::Kind::from(port as u16),
                                            ));

                                            continue;
                                        }
                                        None => {}
                                    }

                                    return Err(ErrorKind::PortValueMustBeStringOrInt(
                                        value.to_owned(),
                                    ));
                                } else {
                                    event!(Level::TRACE, "Port value not found");
                                }
                            }
                            None => return Err(ErrorKind::PortMustBeTable(port.to_owned())),
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

            let paths = match paths.as_table() {
                Some(paths) => paths,
                None => return Err(ErrorKind::PathsMustBeTable(paths.to_owned())),
            };

            match paths.get("acceptable") {
                Some(acceptable) => {
                    event!(Level::TRACE, "Parse acceptable paths");

                    let acceptable = match acceptable.as_array() {
                        Some(acceptable) => acceptable,
                        None => return Err(ErrorKind::PathsMustBeArray(acceptable.to_owned())),
                    };

                    for path in acceptable {
                        match path.as_table() {
                            Some(path) => {
                                if let Some(value) = path.get("value") {
                                    match value.as_str() {
                                        Some(path) => {
                                            route_builder = route_builder.path(path::Matcher::new(
                                                PermissionKind::Acceptable,
                                                path::Kind::try_from(path.to_owned())?,
                                            ));
                                        }
                                        None => {
                                            return Err(ErrorKind::PathValueMustBeString(
                                                value.to_owned(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Path value not found");
                                }
                            }
                            None => return Err(ErrorKind::PathMustBeTable(path.to_owned())),
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

                    let unacceptable = match unacceptable.as_array() {
                        Some(unacceptable) => unacceptable,
                        None => return Err(ErrorKind::PathsMustBeArray(unacceptable.to_owned())),
                    };

                    for path in unacceptable {
                        match path.as_table() {
                            Some(path) => {
                                if let Some(value) = path.get("value") {
                                    match value.as_str() {
                                        Some(path) => {
                                            route_builder = route_builder.path(path::Matcher::new(
                                                PermissionKind::Unacceptable,
                                                path::Kind::try_from(path.to_owned())?,
                                            ));
                                        }
                                        None => {
                                            return Err(ErrorKind::PathValueMustBeString(
                                                value.to_owned(),
                                            ))
                                        }
                                    }
                                } else {
                                    event!(Level::TRACE, "Path value not found");
                                }
                            }
                            None => return Err(ErrorKind::PathMustBeTable(path.to_owned())),
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
            value = "example.com"

            [[routes.hosts.acceptable]]
            value = "example2.com"

            [[routes.hosts.unacceptable]]
            value = "127.0.0.1"

            [[routes.schemes.acceptable]]
            value = "https"

            [[routes.schemes.acceptable]]

            [[routes.schemes.acceptable]]
            value = "http"

            [[routes.schemes.unacceptable]]
            value = "http"

            [[routes.ports.acceptable]]
            value = "8080"

            [[routes.ports.acceptable]]
            value = 80

            [[routes.ports.acceptable]]
            value = "80*"

            [[routes.ports.unacceptable]]

            [[routes.paths.acceptable]]
            value = "/example/"

            [[routes.paths.acceptable]]
            value = "/example2/*"

            [[routes.paths.unacceptable]]
            value = "/admin/*"

            [[routes.paths.unacceptable]]

            [[routes.methods.acceptable]]
            value = "GET"

            [[routes.methods.acceptable]]
            value = "PATCH"

            [[routes.methods.unacceptable]]
            value = "POST"
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
            host::Kind::Exact(Host::Domain("example2.com".to_owned()))
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
