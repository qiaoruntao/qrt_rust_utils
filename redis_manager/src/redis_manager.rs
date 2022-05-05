use deadpool_redis::{Pool, Runtime};

use crate::redis_handler::RedisHandler;

pub struct RedisManager {
    pool: Pool,
}

impl RedisManager {
    pub async fn new(connection_str: &str, auth: Option<&str>) -> RedisManager {
        let config = deadpool_redis::Config::from_url(connection_str);
        let pool = config.create_pool(Some(Runtime::Tokio1)).unwrap();
        if let Some(auth_str) = auth {
            let mut connection = pool.get().await.unwrap();
            deadpool_redis::redis::cmd("AUTH").arg(auth_str).query_async::<_, ()>(&mut connection).await.unwrap();
        }
        RedisManager {
            pool
        }
    }

    pub fn get_handler<T>(&self) -> RedisHandler<T> {
        let pool = self.pool.clone();
        RedisHandler {
            pool,
            phantom: Default::default(),
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
        let password = env::var("redis_password").expect("redis_password not found");
        let manager = RedisManager::new(str.as_str(), Some(password.as_str())).await;
        let handler = manager.get_handler();
        let x1 = handler.set_value("aaa", 111).await;
        dbg!(x1);
        let x: Option<i32> = handler.get_value("aaa").await;
        dbg!(x);
    }
}