pub struct ConsumerConfig {
    pub max_concurrency: u32,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        ConsumerConfig {
            max_concurrency: u32::MAX
        }
    }
}