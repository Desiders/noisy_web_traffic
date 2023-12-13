use super::route::Route;

#[derive(Debug, Default, Clone)]
pub struct Rules {
    pub route: Route,
}

impl Rules {
    pub const fn new(route: Route) -> Self {
        Self { route }
    }

    pub fn builder() -> Builder {
        Builder::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    pub route: Route,
}

impl Builder {
    pub fn route(mut self, route: Route) -> Self {
        self.route = route;
        self
    }

    pub fn build(self) -> Rules {
        Rules::new(self.route)
    }
}
