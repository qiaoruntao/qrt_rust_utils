use std::fmt::Debug;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

pub struct Task<T: Debug + Serialize + DeserializeOwned, R: Debug + Serialize + DeserializeOwned> {
    name: String,
    option: TaskOptions,
    param: T,
    state: R,
}

pub struct TaskOptions {
    time_limit: Duration,
    hard_time_limit: Duration,
    max_retries: u32,
    min_retry_delay: Duration,
    max_retry_delay: Duration,
    retry_for_unexpected: bool,
}

#[cfg(test)]
mod test_task {
    #[test]
    fn send_task() {

    }
}