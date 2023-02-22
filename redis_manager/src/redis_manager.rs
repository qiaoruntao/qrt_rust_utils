use std::sync::Arc;

use deadpool_redis::{Pool, PoolConfig, Runtime};
use log_util::tracing::instrument;

use crate::redis_handler::RedisHandler;

pub struct RedisManager {
    pool: Arc<Pool>,
}

impl RedisManager {
    #[instrument]
    pub async fn new(connection_str: &str) -> RedisManager {
        let mut config = deadpool_redis::Config::from_url(connection_str);
        config.pool = Some(PoolConfig::new(2));
        let pool = config.create_pool(Some(Runtime::Tokio1)).unwrap();
        RedisManager {
            pool: Arc::new(pool)
        }
    }

    #[instrument(skip(self))]
    pub fn get_handler(&self) -> RedisHandler {
        RedisHandler {
            pool: self.pool.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use log_util::init_logger;

    use crate::redis_manager::RedisManager;

    #[tokio::test]
    async fn test_redis() {
        init_logger("test", None);
        let str = env::var("redis_key").expect("redis_key not found");
        let manager = RedisManager::new(str.as_str()).await;
        let handler = manager.get_handler();
        let x1 = handler.set_value("aaa", 111).await;
        dbg!(x1);
        let x: Option<i32> = handler.get_value("aaa").await;
        dbg!(x);
    }
}