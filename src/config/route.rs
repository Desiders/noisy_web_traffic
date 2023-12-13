use super::routes::{
    host, hosts::Hosts, method, methods::Methods, path, paths::Paths, port, ports::Ports, scheme,
    schemes::Schemes,
};

use std::iter;

#[derive(Debug, Default, Clone)]
pub struct Route {
    pub hosts: Hosts,
    pub methods: Methods,
    pub paths: Paths,
    pub ports: Ports,
    pub schemes: Schemes,
}

impl Route {
    pub fn new(
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
            hosts,
            methods,
            paths,
            ports,
            schemes,
        }
    }

    pub fn builder() -> Builder {
        Builder::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    pub hosts: Hosts,
    pub methods: Methods,
    pub paths: Paths,
    pub ports: Ports,
    pub schemes: Schemes,
}

impl Builder {
    pub fn hosts(mut self, hosts: impl IntoIterator<Item = super::routes::host::Matcher>) -> Self {
        self.hosts.extend(hosts);
        self
    }

    pub fn host(mut self, host: super::routes::host::Matcher) -> Self {
        self.hosts.extend(iter::once(host));
        self
    }

    pub fn methods(
        mut self,
        methods: impl IntoIterator<Item = super::routes::method::Matcher>,
    ) -> Self {
        self.methods.extend(methods);
        self
    }

    pub fn method(mut self, method: super::routes::method::Matcher) -> Self {
        self.methods.extend(iter::once(method));
        self
    }

    pub fn paths(mut self, paths: impl IntoIterator<Item = super::routes::path::Matcher>) -> Self {
        self.paths.extend(paths);
        self
    }

    pub fn path(mut self, path: super::routes::path::Matcher) -> Self {
        self.paths.extend(iter::once(path));
        self
    }

    pub fn ports(mut self, ports: impl IntoIterator<Item = super::routes::port::Matcher>) -> Self {
        self.ports.extend(ports);
        self
    }

    pub fn port(mut self, port: super::routes::port::Matcher) -> Self {
        self.ports.extend(iter::once(port));
        self
    }

    pub fn schemes(
        mut self,
        schemes: impl IntoIterator<Item = super::routes::scheme::Matcher>,
    ) -> Self {
        self.schemes.extend(schemes);
        self
    }

    pub fn scheme(mut self, scheme: super::routes::scheme::Matcher) -> Self {
        self.schemes.extend(iter::once(scheme));
        self
    }

    pub fn build(self) -> Route {
        Route::new(
            self.hosts,
            self.methods,
            self.paths,
            self.ports,
            self.schemes,
        )
    }
}
