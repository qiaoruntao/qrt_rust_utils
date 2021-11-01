use std::fmt::Debug;

use mongodb::options::ServerAddress::Tcp;
use mongodb::options::{Credential, Tls, TlsOptions};
use mongodb::{options::ClientOptions, Client};
use mongodb::{Collection, Database};
use serde::de::DeserializeOwned;
use serde::Serialize;

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
        match self.client.list_databases(None, None).await {
            Ok(_) => true,
            Err(_) => false,
        }
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
}

#[cfg(test)]
mod mongodb_manager_test {
    use crate::config_manage::config_manager::ConfigManager;
    use crate::mongodb_manager::entity::mongodb_config::MongoDbConfig;
    use crate::mongodb_manager::mongodb_manager::MongoDbManager;

    #[tokio::test]
    async fn basic() -> Result<(), Box<dyn std::error::Error>> {
        let config: MongoDbConfig =
            ConfigManager::read_config_with_directory("config/mongo").unwrap();
        let manager = MongoDbManager::new(config, "ts_test").unwrap();
        assert_eq!(manager.ping().await, true);
        Ok(())
    }
}
