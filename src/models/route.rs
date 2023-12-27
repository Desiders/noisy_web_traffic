use crate::models::routes::{
    follow_robots_exclusion_protocol::FollowRobotsExclusionProtocol, host, hosts::Hosts, path,
    paths::Paths, port, ports::Ports, root_url, root_urls::RootUrls, scheme, schemes::Schemes,
};

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use super::routes::follow_robots_exclusion_protocol;

#[derive(Debug, Clone)]
pub struct Route {
    pub root_urls: RootUrls,
    pub follow_robots_exclusion_protocol: FollowRobotsExclusionProtocol,
    pub hosts: Hosts,
    pub paths: Paths,
    pub ports: Ports,
    pub schemes: Schemes,
}

impl Route {
    pub fn new(
        root_urls: RootUrls,
        follow_robots_exclusion_protocol: FollowRobotsExclusionProtocol,
        mut hosts: Hosts,
        mut paths: Paths,
        mut ports: Ports,
        mut schemes: Schemes,
    ) -> Self {
        if hosts.acceptable.is_empty() {
            hosts.acceptable.push(host::Kind::Any);
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
            follow_robots_exclusion_protocol,
            hosts,
            paths,
            ports,
            schemes,
        }
    }

    pub fn host_matches(&self, host: impl AsRef<str>) -> bool {
        self.hosts.matches(host)
    }

    pub fn path_matches(&self, path: impl AsRef<str>) -> bool {
        self.paths.matches(path)
    }

    pub fn port_matches(&self, port: u16) -> bool {
        self.ports.matches(port)
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
            "Route {{ root_urls: {}, hosts: {}, paths: {}, ports: {}, schemes: {} }}",
            self.root_urls, self.hosts, self.paths, self.ports, self.schemes,
        )
    }
}

impl Default for Route {
    fn default() -> Self {
        Self::new(
            RootUrls::default(),
            follow_robots_exclusion_protocol::FollowRobotsExclusionProtocol::default(),
            Hosts::default(),
            Paths::default(),
            Ports::default(),
            Schemes::default(),
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    root_urls: RootUrls,
    follow_robots_exclusion_protocol: FollowRobotsExclusionProtocol,
    hosts: Hosts,
    paths: Paths,
    ports: Ports,
    schemes: Schemes,
}

impl Builder {
    pub fn root_url(mut self, root_url: root_url::RootUrl) -> Self {
        self.root_urls.extend(iter::once(root_url));
        self
    }

    pub fn follow_robots_exclusion_protocol(
        mut self,
        follow_robots_exclusion_protocol: follow_robots_exclusion_protocol::FollowRobotsExclusionProtocol,
    ) -> Self {
        self.follow_robots_exclusion_protocol = follow_robots_exclusion_protocol;
        self
    }

    pub fn host(mut self, host: host::Matcher) -> Self {
        self.hosts.extend(iter::once(host));
        self
    }

    pub fn path(mut self, path: path::Matcher) -> Self {
        self.paths.extend(iter::once(path));
        self
    }

    pub fn port(mut self, port: port::Matcher) -> Self {
        self.ports.extend(iter::once(port));
        self
    }

    pub fn scheme(mut self, scheme: scheme::Matcher) -> Self {
        self.schemes.extend(iter::once(scheme));
        self
    }

    pub fn build(self) -> Route {
        Route::new(
            self.root_urls,
            self.follow_robots_exclusion_protocol,
            self.hosts,
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
            .follow_robots_exclusion_protocol(
                follow_robots_exclusion_protocol::FollowRobotsExclusionProtocol::new(false),
            )
            .host(host::Matcher::new(
                PermissionKind::Acceptable,
                host::Kind::exact("example.com").unwrap(),
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

        assert_eq!(
            route.follow_robots_exclusion_protocol,
            follow_robots_exclusion_protocol::FollowRobotsExclusionProtocol::new(false)
        );

        assert_eq!(route.hosts.acceptable.len(), 1);
        assert_eq!(
            route.hosts.acceptable[0],
            host::Kind::exact("example.com").unwrap()
        );

        assert_eq!(route.paths.acceptable.len(), 1);
        assert_eq!(route.paths.acceptable[0], path::Kind::exact("/"));

        assert_eq!(route.ports.acceptable.len(), 1);
        assert_eq!(route.ports.acceptable[0], port::Kind::exact(80));

        assert_eq!(route.schemes.acceptable.len(), 1);
        assert_eq!(route.schemes.acceptable[0], scheme::Kind::Http);

        let route = Route::builder().build();

        assert_eq!(route.hosts.acceptable.len(), 1);
        assert_eq!(route.hosts.acceptable[0], host::Kind::Any);

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
    }
}
