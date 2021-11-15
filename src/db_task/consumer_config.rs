use std::time::Duration;

pub struct ConsumerConfig {
    pub max_concurrency: u32,
    pub task_delay: Duration,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        ConsumerConfig {
            max_concurrency: u32::MAX,
            task_delay: Duration::ZERO,
        }
    }
}