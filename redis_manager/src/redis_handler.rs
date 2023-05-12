use std::sync::Arc;

use deadpool_redis::{Connection, Pool};
use deadpool_redis::redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use deadpool_redis::redis::aio::PubSub;
use deadpool_redis::redis::RedisResult;
use qrt_log_utils::tracing::error;

pub struct RedisHandler {
    pub pool: Arc<Pool>,
}

impl RedisHandler {
    pub async fn set_ex<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, value: T, seconds: usize) -> bool {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return false;
            }
        };
        connection.set_ex(key, value, seconds).await.unwrap_or(false)
    }
    pub async fn set_value<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, value: T) -> bool {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return false;
            }
        };
        connection.set(key, value).await.unwrap_or(false)
    }

    pub async fn set_expire(&self, key: &str, seconds: usize) -> bool {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return false;
            }
        };
        connection.expire(key, seconds).await.unwrap_or(false)
    }

    pub async fn push_value<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, value: T) -> bool {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return false;
            }
        };
        connection.rpush(key, value).await.unwrap_or(false)
    }

    pub async fn get_value<T: ToRedisArgs + Sync + Send + FromRedisValue, K: ToRedisArgs + Send + Sync>(&self, key: K) -> Option<T> {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return None;
            }
        };
        match connection.get(key).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }

    pub async fn fetch_list<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str) -> Option<Vec<T>> {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return None;
            }
        };
        match connection.lrange(key, 0, isize::MAX).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }

    pub async fn fetch_map<T: ToRedisArgs + Sync + Send + FromRedisValue, K: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str) -> Option<Vec<(T, K)>> {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return None;
            }
        };
        match connection.hgetall(key).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }

    pub async fn fetch_hkey<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, field: &str) -> Option<T> {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return None;
            }
        };
        match connection.hget(key, field).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }

    pub async fn del_hkey(&self, key: &str, field: &str) -> Option<i64> {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return None;
            }
        };
        match connection.hdel(key, field).await {
            Ok(v) => v,
            Err(e) => {
                error!("{}",&e);
                None
            }
        }
    }

    pub async fn set_nx_expire(&self, key: &str, value: &str, expire_ms: i64) -> bool {
        let mut connection = match self.pool.get().await {
            Ok(v) => { v }
            Err(e) => {
                error!("failed to get redis connection {}",e);
                return false;
            }
        };
        let redis_result: RedisResult<Option<String>> = deadpool_redis::redis::cmd("SET")
            .arg(key).arg(value).arg("NX").arg("PX").arg(expire_ms)
            .query_async(&mut connection).await;
        match redis_result {
            Ok(Some(v)) => {
                v == "OK"
            }
            Ok(None) => false,
            Err(e) => {
                error!("{}",&e);
                false
            }
        }
    }

    pub async fn get_pubsub(&self) -> Option<PubSub> {
        // TODO: is this necessary?
        // self.pool.resize(self.pool.status().max_size + 1);
        match self.pool.get().await {
            Ok(connection) => {
                let deadpool_connection = Connection::take(connection);
                Some(deadpool_connection.into_pubsub())
            }
            Err(e) => {
                error!("failed to get pubsub {}",e);
                None
            }
        }
    }
}

#[cfg(test)]
mod test_redis {
    use std::env;
    use std::time::Duration;

    use futures::StreamExt;

    use crate::redis_manager::RedisManager;

    #[tokio::test]
    async fn test_list() {
        let str = env::var("redis_key").expect("redis_key not found");
        let redis_manager = RedisManager::new(str.as_str()).await;
        let handler = redis_manager.get_handler();
        let option = handler.get_value::<String, _>("did").await;
        dbg!(option);
        let option = handler.set_value::<String>("did", "aaa".into()).await;
        dbg!(option);
        let option = handler.get_value::<String, _>("did_").await;
        dbg!(option);
        let option = handler.fetch_list::<i64>("BanList").await;
        dbg!(option);
        let option = handler.get_value::<Vec<Option<String>>, _>(&["did", "did", "did", "did2"]).await;
        dbg!(option);
        let option: Option<Vec<(i64, i64)>> = handler.fetch_map("Broadcasting").await;
        dbg!(option);
    }

    #[tokio::test]
    async fn test_pubsub_creation() {
        let str = env::var("redis_key").expect("redis_key not found");
        let redis_manager = RedisManager::new(str.as_str()).await;
        let redis_handler = redis_manager.get_handler();
        for _ in 0..100 {
            let mut pub_sub = redis_handler.get_pubsub().await.unwrap();
            pub_sub.psubscribe("__key*__:Broadcasting").await.expect("failed to listen to pubsub");
            // drop(pub_sub);
        }
    }

    #[tokio::test]
    async fn test_pubsub() {
        let str = env::var("redis_key").expect("redis_key not found");
        let redis_manager = RedisManager::new(str.as_str()).await;
        let redis_handler = redis_manager.get_handler();
        let mut pubsub = match redis_handler.get_pubsub().await {
            None => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                return;
            }
            Some(v) => { v }
        };
        pubsub.psubscribe("__key*__:Broadcasting").await.expect("failed to listen to pubsub");
        StreamExt::for_each(pubsub.on_message().take(5), |msg| {
            async move {
                dbg!(&msg);
                // TODO:
                // let channel = msg.get_channel_name();
                // let str: String = msg.get_payload().unwrap();
                // println!("{}={}", channel, str);
            }
        }).await;
    }

    #[tokio::test]
    async fn test_set_nx_ex() {
        let str = env::var("redis_key").expect("redis_key not found");
        let redis_manager = RedisManager::new(str.as_str()).await;
        let redis_handler = redis_manager.get_handler();
        redis_handler.set_nx_expire("a", "b", 99999).await;
    }
}