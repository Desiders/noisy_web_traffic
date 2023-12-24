use crate::models::routes::{
    host, hosts::Hosts, method, methods::Methods, path, paths::Paths, port, ports::Ports, root_url,
    root_urls::RootUrls, scheme, schemes::Schemes,
};

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

#[derive(Debug, Clone)]
pub struct Route {
    pub root_urls: RootUrls,
    pub hosts: Hosts,
    pub methods: Methods,
    pub paths: Paths,
    pub ports: Ports,
    pub schemes: Schemes,
}

impl Route {
    pub fn new(
        root_urls: RootUrls,
        mut hosts: Hosts,
        mut methods: Methods,
        mut paths: Paths,
        mut ports: Ports,
        mut schemes: Schemes,
    ) -> Self {
        if hosts.acceptable.is_empty() {
            hosts.acceptable.push(host::Kind::Any);
        }

        if methods.acceptable.is_empty() {
            methods.acceptable.push(method::Kind::AnySupported);
        }

        if paths.acceptable.is_empty() {
            paths.acceptable.push(path::Kind::Any);
        }

        if ports.acceptable.is_empty() {
            ports.acceptable.push(port::Kind::Any);
        }

        if schemes.acceptable.is_empty() {
            schemes.acceptable.push(scheme::Kind::AnySupported);
        }

        Self {
            root_urls,
            hosts,
            methods,
            paths,
            ports,
            schemes,
        }
    }

    pub fn host_matches(&self, host: impl AsRef<str>) -> bool {
        self.hosts.matches(host)
    }

    pub fn method_matches(&self, method: impl AsRef<str>) -> bool {
        self.methods.matches(method)
    }

    pub fn path_matches(&self, path: impl AsRef<str>) -> bool {
        self.paths.matches(path)
    }

    pub fn port_matches(&self, port: u16) -> bool {
        self.ports.matches(port)
    }

    pub fn port_matches_str(&self, port: impl AsRef<str>) -> bool {
        self.ports.matches_str(port)
    }

    pub fn scheme_matches(&self, scheme: impl AsRef<str>) -> bool {
        self.schemes.matches(scheme)
    }

    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Route {{ root_urls: {}, hosts: {}, methods: {}, paths: {}, ports: {}, schemes: {} }}",
            self.root_urls, self.hosts, self.methods, self.paths, self.ports, self.schemes,
        )
    }
}

impl Default for Route {
    fn default() -> Self {
        Self::new(
            RootUrls::default(),
            Hosts::default(),
            Methods::default(),
            Paths::default(),
            Ports::default(),
            Schemes::default(),
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    root_urls: RootUrls,
    hosts: Hosts,
    methods: Methods,
    paths: Paths,
    ports: Ports,
    schemes: Schemes,
}

impl Builder {
    pub fn root_urls(mut self, root_urls: impl IntoIterator<Item = root_url::RootUrl>) -> Self {
        self.root_urls.extend(root_urls);
        self
    }

    pub fn root_url(mut self, root_url: root_url::RootUrl) -> Self {
        self.root_urls.extend(iter::once(root_url));
        self
    }

    pub fn hosts(mut self, hosts: impl IntoIterator<Item = host::Matcher>) -> Self {
        self.hosts.extend(hosts);
        self
    }

    pub fn host(mut self, host: host::Matcher) -> Self {
        self.hosts.extend(iter::once(host));
        self
    }

    pub fn methods(mut self, methods: impl IntoIterator<Item = method::Matcher>) -> Self {
        self.methods.extend(methods);
        self
    }

    pub fn method(mut self, method: method::Matcher) -> Self {
        self.methods.extend(iter::once(method));
        self
    }

    pub fn paths(mut self, paths: impl IntoIterator<Item = path::Matcher>) -> Self {
        self.paths.extend(paths);
        self
    }

    pub fn path(mut self, path: path::Matcher) -> Self {
        self.paths.extend(iter::once(path));
        self
    }

    pub fn ports(mut self, ports: impl IntoIterator<Item = port::Matcher>) -> Self {
        self.ports.extend(ports);
        self
    }

    pub fn port(mut self, port: port::Matcher) -> Self {
        self.ports.extend(iter::once(port));
        self
    }

    pub fn schemes(mut self, schemes: impl IntoIterator<Item = scheme::Matcher>) -> Self {
        self.schemes.extend(schemes);
        self
    }

    pub fn scheme(mut self, scheme: scheme::Matcher) -> Self {
        self.schemes.extend(iter::once(scheme));
        self
    }

    pub fn build(self) -> Route {
        Route::new(
            self.root_urls,
            self.hosts,
            self.methods,
            self.paths,
            self.ports,
            self.schemes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::routes::permission::Kind as PermissionKind;

    #[test]
    fn test_route_builder() {
        let route = Route::builder()
            .root_url(root_url::RootUrl::new("https://example.com").unwrap())
            .host(host::Matcher::new(
                PermissionKind::Acceptable,
                host::Kind::exact("example.com").unwrap(),
            ))
            .method(method::Matcher::new(
                PermissionKind::Acceptable,
                method::Kind::Get,
            ))
            .path(path::Matcher::new(
                PermissionKind::Acceptable,
                path::Kind::exact("/"),
            ))
            .port(port::Matcher::new(
                PermissionKind::Acceptable,
                port::Kind::exact(80),
            ))
            .scheme(scheme::Matcher::new(
                PermissionKind::Acceptable,
                scheme::Kind::Http,
            ))
            .build();

        assert_eq!(route.root_urls.len(), 1);
        assert_eq!(
            route.root_urls[0],
            root_url::RootUrl::new("https://example.com").unwrap()
        );

        assert_eq!(route.hosts.acceptable.len(), 1);
        assert_eq!(
            route.hosts.acceptable[0],
            host::Kind::exact("example.com").unwrap()
        );

        assert_eq!(route.methods.acceptable.len(), 1);
        assert_eq!(route.methods.acceptable[0], method::Kind::Get);

        assert_eq!(route.paths.acceptable.len(), 1);
        assert_eq!(route.paths.acceptable[0], path::Kind::exact("/"));

        assert_eq!(route.ports.acceptable.len(), 1);
        assert_eq!(route.ports.acceptable[0], port::Kind::exact(80));

        assert_eq!(route.schemes.acceptable.len(), 1);
        assert_eq!(route.schemes.acceptable[0], scheme::Kind::Http);

        let route = Route::builder().build();

        assert_eq!(route.hosts.acceptable.len(), 1);
        assert_eq!(route.hosts.acceptable[0], host::Kind::Any);

        assert_eq!(route.methods.acceptable.len(), 1);
        assert_eq!(route.methods.acceptable[0], method::Kind::AnySupported);

        assert_eq!(route.paths.acceptable.len(), 1);
        assert_eq!(route.paths.acceptable[0], path::Kind::Any);

        assert_eq!(route.ports.acceptable.len(), 1);
        assert_eq!(route.ports.acceptable[0], port::Kind::Any);

        assert_eq!(route.schemes.acceptable.len(), 1);
        assert_eq!(route.schemes.acceptable[0], scheme::Kind::AnySupported);

        let route = Route::builder()
            .host(host::Matcher::new(
                PermissionKind::Acceptable,
                host::Kind::exact("example.com").unwrap(),
            ))
            .host(host::Matcher::new(
                PermissionKind::Unacceptable,
                host::Kind::exact("example2.org").unwrap(),
            ))
            .build();

        assert_eq!(route.hosts.acceptable.len(), 1);
        assert_eq!(route.hosts.unacceptable.len(), 1);
        assert_eq!(
            route.hosts.acceptable[0],
            host::Kind::exact("example.com").unwrap()
        );
        assert_eq!(
            route.hosts.unacceptable[0],
            host::Kind::exact("example2.org").unwrap()
        );

        let route = Route::builder()
            .hosts([
                host::Matcher::new(
                    PermissionKind::Acceptable,
                    host::Kind::exact("example.com").unwrap(),
                ),
                host::Matcher::new(
                    PermissionKind::Unacceptable,
                    host::Kind::exact("example2.org").unwrap(),
                ),
            ])
            .build();

        assert_eq!(route.hosts.acceptable.len(), 1);
        assert_eq!(route.hosts.unacceptable.len(), 1);
        assert_eq!(
            route.hosts.acceptable[0],
            host::Kind::exact("example.com").unwrap()
        );
        assert_eq!(
            route.hosts.unacceptable[0],
            host::Kind::exact("example2.org").unwrap()
        );
    }
}
