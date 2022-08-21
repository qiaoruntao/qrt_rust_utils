use std::marker::PhantomData;
use std::num::NonZeroUsize;

use deadpool_redis::Pool;
use deadpool_redis::redis::{AsyncCommands, FromRedisValue, ToRedisArgs};

use log_util::tracing::error;

pub struct RedisHandler<T> {
    pub pool: Pool,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: ToRedisArgs + Sync + Send + FromRedisValue> RedisHandler<T> {
    pub async fn set_ex(&self, key: &str, value: T, seconds: usize) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.set_ex(key, value, seconds).await.unwrap()
    }
    pub async fn set_value(&self, key: &str, value: T) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.set(key, value).await.unwrap()
    }

    pub async fn set_expire(&self, key: &str, seconds: usize) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.expire(key, seconds).await.unwrap()
    }

    pub async fn push_value(&self, key: &str, value: T) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.rpush(key, value).await.unwrap()
    }

    pub async fn get_value(&self, key: &str) -> Option<T> {
        let mut connection = self.pool.get().await.unwrap();
        match connection.get(key).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }

    pub async fn fetch_list_value(&self, key: &str) -> Option<T> {
        let mut connection = self.pool.get().await.unwrap();
        match connection.lpop(key, Some(NonZeroUsize::new(1).unwrap())).await {
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

    use crate::redis_handler::RedisHandler;
    use crate::redis_manager::RedisManager;

    #[tokio::test]
    async fn test_list() {
        let str = env::var("redis_key").expect("redis_key not found");
        let password = env::var("redis_password").expect("redis_password not found");
        let redis_manager = RedisManager::new(str.as_str()).await;
        let handler: RedisHandler<String> = redis_manager.get_handler();
        let option = handler.get_value("did").await;
        dbg!(option);
        let option = handler.set_value("did", "aaa".into()).await;
        dbg!(option);
        let option = handler.get_value("did_").await;
        dbg!(option);
    }
}