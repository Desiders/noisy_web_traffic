use rand::{thread_rng, Rng as _};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Time {
    pub min_sleep_between_requests: u64,
    pub max_sleep_between_requests: u64,
    pub request_timeout: u64,
}

impl Time {
    pub const fn new(
        min_sleep_between_requests: u64,
        max_sleep_between_requests: u64,
        request_timeout: u64,
    ) -> Self {
        Self {
            min_sleep_between_requests,
            max_sleep_between_requests,
            request_timeout,
        }
    }

    pub fn get_random_sleep_between_requests_raw(&self) -> u64 {
        let mut rng = thread_rng();

        rng.gen_range(self.min_sleep_between_requests..=self.max_sleep_between_requests)
    }

    pub fn get_random_sleep_between_requests(&self) -> Duration {
        Duration::from_millis(self.get_random_sleep_between_requests_raw())
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new(3000, 60000, 7000)
    }
}
