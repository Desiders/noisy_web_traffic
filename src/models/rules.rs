use super::{polling::Polling, route::Route};

use std::fmt::{self, Display, Formatter};

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

impl Display for Rules {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rules {{ route: {}, polling: {} }}",
            self.route, self.polling
        )
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
