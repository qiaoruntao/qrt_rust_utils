use std::sync::{Arc, Mutex};

use deadpool_redis::Pool;
use deadpool_redis::redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use redis::aio::PubSub;

use log_util::tracing::{error, warn};

pub struct RedisHandler {
    pub pool: Arc<Pool>,
    pub pubsub: Arc<Mutex<Option<redis::aio::PubSub>>>,
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

    pub async fn set_expire(&self, key: &str, seconds: usize) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.expire(key, seconds).await.unwrap()
    }

    pub async fn push_value<T: ToRedisArgs + Sync + Send + FromRedisValue>(&self, key: &str, value: T) -> bool {
        let mut connection = self.pool.get().await.unwrap();
        connection.rpush(key, value).await.unwrap()
    }

    pub async fn get_value<T: ToRedisArgs + Sync + Send + FromRedisValue, K: ToRedisArgs + Send + Sync>(&self, key: K) -> Option<T> {
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

    pub async fn get_pubsub(&self) -> Arc<Mutex<Option<PubSub>>> {
        let mut guard = self.pubsub.lock().unwrap();
        if guard.is_none() {
            let obj = deadpool_redis::Connection::take(self.pool.get().await.unwrap());
            let pubsub = obj.into_pubsub();
            *guard = Some(pubsub);
        }
        self.pubsub.clone()
    }

    pub async fn p_subscribe(&self, pattern: &str) {
        let x1 = self.get_pubsub().await;
        let mut mutex_guard = x1.lock().unwrap();
        if let Some(pubsub) = mutex_guard.as_mut() {
            pubsub.psubscribe(pattern).await.unwrap();
            // pubsub.psubscribe("__key*__:*").await.unwrap();
        } else {
            warn!("failed to fetch pubsub");
        }
        //
        // let x = pubsub.on_message();
        // tokio::spawn(async move {
        //     futures::StreamExt::for_each(x.take(1), |msg| {
        //         async move {
        //             let channel = msg.get_channel_name();
        //             let str: String = msg.get_payload().unwrap();
        //             println!("{}={}", channel, str);
        //         }
        //     }).await;
        // });
        //
        //
        // futures::StreamExt::for_each(pubsub.on_message().take(1), |msg| {
        //     async move {
        //         let channel = msg.get_channel_name();
        //         let str: String = msg.get_payload().unwrap();
        //         println!("{}={}", channel, str);
        //     }
        // }).await;
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
    }

    #[tokio::test]
    async fn test_pubsub() {
        let str = env::var("redis_key").expect("redis_key not found");
        let redis_manager = RedisManager::new(str.as_str()).await;
        let handler = redis_manager.get_handler();
        // handler.subscribe().await;
    }
}