use std::time::Duration;

use anyhow::anyhow;
use futures::StreamExt;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{doc, Document};
use mongodb::options::{ClientOptions, InsertOneOptions, WriteConcern};
use mongodb::results::InsertOneResult;
use serde::Serialize;
use serde::de::DeserializeOwned;

use log_util::tracing::error;

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
    pub fn get_collection<T: Serialize>(&self, collection_name: &str) -> Collection<T> {
        self.database.collection(collection_name)
    }
    pub async fn save_collection<T: Serialize>(&self, collection_name: &str, obj: &T) -> anyhow::Result<InsertOneResult> {
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
    pub async fn aggregate_one<T: Serialize + DeserializeOwned>(&self, collection_name: &str, pipeline: impl IntoIterator<Item=Document>) -> anyhow::Result<T> {
        let collection = self.get_collection::<Document>(collection_name);
        let find_result = collection.aggregate(pipeline, None).await;
        if let Err(e) = find_result {
            error!("cursor find error {:?}",&e);
            return Err(e.into());
        }
        let mut cursor = find_result.unwrap();
        let next = cursor.next().await;
        return match next {
            None => {
                Err(anyhow!("not found"))
            }
            Some(Err(e)) => {
                Err(anyhow!(e))
            }
            Some(Ok(value)) => {
                match mongodb::bson::from_document(value) {
                    Ok(value) => {
                        Ok(value)
                    }
                    Err(e) => {
                        Err(anyhow!(e))
                    }
                }
            }
        };
    }
}