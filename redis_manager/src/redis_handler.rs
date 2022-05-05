use std::marker::PhantomData;

use deadpool_redis::Pool;
use deadpool_redis::redis::{AsyncCommands, FromRedisValue, ToRedisArgs};

use log_util::tracing::error;

pub struct RedisHandler<T> {
    pub pool: Pool,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: ToRedisArgs + Sync + Send + FromRedisValue> RedisHandler<T> {
    pub async fn set_value(&self, key: &str, value: T) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.set(key, value).await.unwrap()
    }
    pub async fn get_value(&self, key: &str) -> Option<T> {
        let mut connection = self.pool.get().await.unwrap();
        match connection.get(key).await {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }
}