use std::time::Duration;

use mongodb::{Client, Collection, Database};
use mongodb::bson::{doc, Document};
use mongodb::options::{ClientOptions, InsertOneOptions, WriteConcern};
use mongodb::results::InsertOneResult;
use serde::Serialize;

pub struct MongodbSaver {
    database: Database,
}

impl MongodbSaver {
    pub async fn init(connection_str: &str) -> Self {
        let client_options = ClientOptions::parse(connection_str).await.unwrap();
        let target_database = client_options.default_database.clone().unwrap();
        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();
        let database = client.database(target_database.as_str());
        MongodbSaver {
            database
        }
    }
    pub async fn save_collection<T: Serialize>(&self, collection_name: &str, obj: &T) -> anyhow::Result<InsertOneResult> {
        let collection: Collection<Document> = self.database.collection(collection_name);
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