use std::sync::Arc;

use deadpool_redis::Pool;
use deadpool_redis::redis::{AsyncCommands, FromRedisValue, ToRedisArgs};

use log_util::tracing::error;

pub struct RedisHandler {
    pub pool: Arc<Pool>,
}

impl RedisHandler {
    pub async fn set_ex<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, value: T, seconds: usize) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.set_ex(key, value, seconds).await.unwrap()
    }
    pub async fn set_value<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, value: T) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.set(key, value).await.unwrap()
    }

    pub async fn set_expire<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, seconds: usize) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.expire(key, seconds).await.unwrap()
    }

    pub async fn push_value<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, value: T) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.rpush(key, value).await.unwrap()
    }

    pub async fn get_value<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str) -> Option<T> {
        let mut connection = self.pool.get().await.unwrap();
        match connection.get(key).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }

    pub async fn fetch_list<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str) -> Option<Vec<T>> {
        let mut connection = self.pool.get().await.unwrap();
        match connection.lrange(key, 0, isize::MAX).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }
}

#[cfg(test)]
mod test_redis_list {
    use std::env;

    use crate::redis_manager::RedisManager;

    #[tokio::test]
    async fn test_list() {
        let str = env::var("redis_key").expect("redis_key not found");
        let redis_manager = RedisManager::new(str.as_str()).await;
        let handler = redis_manager.get_handler();
        let option = handler.get_value::<String>("did").await;
        dbg!(option);
        let option = handler.set_value::<String>("did", "aaa".into()).await;
        dbg!(option);
        let option = handler.get_value::<String>("did_").await;
        dbg!(option);
        let option = handler.fetch_list::<i64>("BanList").await;
        dbg!(option);
    }
}