use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use mongodb::{Client, options::ClientOptions};
use mongodb::{Collection, Database};
use mongodb::bson::{doc, Document};
use mongodb::options::{Credential, InsertOneOptions, Tls, TlsOptions, WriteConcern};
use mongodb::options::ReadConcernLevel::Local;
use mongodb::options::ServerAddress::Tcp;
use mongodb::results::InsertOneResult;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Error, Value};
use tokio::sync::RwLock;
use tracing::info;

use crate::mongodb_manager::entity::mongodb_config::MongoDbConfig;

pub struct MongoDbManager {
    pub client: Client,
    db: Database,
}

// init logic
impl MongoDbManager {
    pub fn new(
        config: MongoDbConfig,
        db_name: &str,
    ) -> Result<MongoDbManager, Box<dyn std::error::Error>> {
        // build option
        let options = MongoDbManager::build_client_options(config);
        // connect to db
        let client = Client::with_options(options)?;
        let db = client.database(db_name);
        Ok(MongoDbManager { client, db })
    }

    fn build_client_options(config: MongoDbConfig) -> ClientOptions {
        let mut options = ClientOptions::default();
        options.hosts = vec![Tcp {
            host: config.address,
            port: Some(config.port),
        }];
        let mut t = Credential::default();
        t.username = Some(config.username);
        t.password = Some(config.password);
        t.source = Some(config.source);
        options.credential = Some(t);
        if config.use_tls {
            let tls_options = TlsOptions::default();
            options.tls = Some(Tls::Enabled(tls_options));
        } else {
            options.tls = None;
        }

        options
    }

    // list databases to check whether we are connected to server
    pub async fn ping(self) -> bool {
        self.client.list_databases(None, None).await.is_ok()
    }
}

// collections
impl MongoDbManager {
    pub fn get_collection<K>(&self, collection_name: &str) -> Collection<K>
        where
            K: Serialize + DeserializeOwned + Unpin + Debug,
    {
        self.db.collection::<K>(collection_name)
    }

    pub async fn save2collection<T>(&self, obj: &T, collection_name: &str) -> Result<InsertOneResult, Box<dyn std::error::Error>>
        where T: Serialize {
        // info!("try inserting data");
        let collection: Collection<Document> = self.get_collection(collection_name);
        let insert_one_options = {
            let mut temp_write_concern = WriteConcern::default();
            temp_write_concern.w_timeout = Some(Duration::from_secs(3));

            let mut temp = InsertOneOptions::default();
            temp.write_concern = Option::from(temp_write_concern);
            temp
        };
        let result = mongodb::bson::to_bson(obj)?;
        let now = chrono::Local::now();
        let document = doc! {"time":now, "data":&result};
        match collection.insert_one(&document, Some(insert_one_options)).await {
            Ok(val) => {
                Ok(val)
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod mongodb_manager_test {
    use std::error::Error;

    use crate::config_manage::config_manager::ConfigManager;
    use crate::mongodb_manager::entity::mongodb_config::MongoDbConfig;
    use crate::mongodb_manager::mongodb_manager::MongoDbManager;

    #[tokio::test]
    async fn basic() -> Result<(), Box<dyn std::error::Error>> {
        let config: MongoDbConfig =
            ConfigManager::read_config_with_directory("config/mongo").unwrap();
        let manager = MongoDbManager::new(config, "ts_test").unwrap();
        assert!(manager.ping().await);
        Ok(())
    }

    #[tokio::test]
    async fn save2collection() -> Result<(), Box<dyn Error>> {
        let config: MongoDbConfig =
            ConfigManager::read_config_with_directory("config/mongo")?;
        let manager = MongoDbManager::new(config, "Logger")?;
        match manager.save2collection(&1, "test_save").await {
            Ok(val) => {
                dbg!(val);
                Ok(())
            }
            Err(e) => {
                dbg!(&e);
                Err(e)
            }
        }
    }
}
