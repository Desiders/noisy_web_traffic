use super::{polling::Polling, route::Route};

#[derive(Debug, Default, Clone)]
pub struct Rules {
    pub route: Route,
    pub polling: Polling,
}

impl Rules {
    pub const fn new(route: Route, polling: Polling) -> Self {
        Self { route, polling }
    }

    pub fn builder() -> Builder {
        Builder::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    pub route: Route,
    pub polling: Polling,
}

impl Builder {
    pub fn route(mut self, route: Route) -> Self {
        self.route = route;
        self
    }

    pub fn polling(mut self, polling: Polling) -> Self {
        self.polling = polling;
        self
    }

    pub fn build(self) -> Rules {
        Rules::new(self.route, self.polling)
    }
}
