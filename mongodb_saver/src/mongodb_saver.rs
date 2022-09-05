use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use futures::StreamExt;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{doc, Document};
use mongodb::options::{ClientOptions, InsertOneOptions, WriteConcern};
use mongodb::results::InsertOneResult;
use rusqlite::Connection;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct MongodbSaver {
    database: Database,
    split_conn: Option<Arc<Connection>>,
}

#[derive(Debug)]
struct RowData {
    id: i32,
    collection_name: String,
    data: String,
}

impl MongodbSaver {
    pub async fn init(connection_str: &str) -> Self {
        let client_options = ClientOptions::parse(connection_str).await.unwrap();
        let target_database = client_options.default_database.clone().unwrap();
        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();
        let database = client.database(target_database.as_str());
        // init split database
        let sqlit_path = env::var("SqlitPath").unwrap_or("./sqlit_temp.sqlit".into());
        let split_conn = {
            match Connection::open(&sqlit_path) {
                Ok(conn) => {
                    if let Err(e) = conn.execute(
                        "CREATE TABLE saved (id INTEGER PRIMARY KEY AUTOINCREMENT, collection_name  TEXT NOT NULL, data  TEXT NOT NULL)",
                        (), // empty list of parameters.
                    ) {
                        eprintln!("{}", e);
                    }
                    Some(Arc::new(conn))
                }
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            }
        };

        MongodbSaver {
            database,
            split_conn,
        }
    }

    pub fn get_collection<T: Serialize>(&self, collection_name: &str) -> Collection<T> {
        self.database.collection(collection_name)
    }

    pub async fn save_collection<T: Serialize>(&self, collection_name: &str, obj: &T) -> anyhow::Result<InsertOneResult> {
        let result = mongodb::bson::to_bson(obj)?;
        let now = chrono::Local::now();
        let document = doc! {"time":now, "data":&result};
        self.save_collection_inner(collection_name, &document).await
    }

    async fn save_collection_inner(&self, collection_name: &str, full_document: &Document) -> anyhow::Result<InsertOneResult> {
        let collection: Collection<Document> = self.get_collection(collection_name);
        let insert_one_options = {
            let mut temp_write_concern = WriteConcern::default();
            temp_write_concern.w_timeout = Some(Duration::from_secs(3));

            let mut temp = InsertOneOptions::default();
            temp.write_concern = Option::from(temp_write_concern);
            temp
        };
        match collection.insert_one(full_document, Some(insert_one_options)).await {
            Ok(val) => {
                Ok(val)
            }
            Err(e) => {
                self.write_local(collection_name, full_document).await;
                Err(e.into())
            }
        }
    }

    pub async fn aggregate_one<T: Serialize + DeserializeOwned>(&self, collection_name: &str, pipeline: impl IntoIterator<Item=Document>) -> anyhow::Result<T> {
        let collection = self.get_collection::<Document>(collection_name);
        let find_result = collection.aggregate(pipeline, None).await;
        if let Err(e) = find_result {
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

    pub async fn write_local(&self, collection_name: &str, document: &Document) {
        let conn = match self.get_sqlit_connection() {
            None => {
                return;
            }
            Some(conn) => { conn }
        };

        let data = RowData {
            id: 0,
            collection_name: collection_name.to_string(),
            data: serde_json::to_string(&document).unwrap(),
        };
        match conn.execute(
            "INSERT INTO saved (collection_name, data) VALUES (?1, ?2)",
            (&data.collection_name, &data.data),
        ) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    pub fn get_sqlit_connection(&self) -> Option<&Arc<Connection>> {
        return self.split_conn.as_ref();
    }

    pub async fn pop_local(&self) {
        let conn = match self.get_sqlit_connection() {
            None => {
                return;
            }
            Some(value) => {
                value
            }
        };
        let mut statement = match conn.prepare("SELECT id, collection_name, data FROM saved") {
            Ok(cursor) => {
                cursor
            }
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        let cursor = statement
            .query_map([], |row| {
                Ok(RowData {
                    id: row.get(0)?,
                    collection_name: row.get(1)?,
                    data: row.get(2)?,
                })
            })
            .unwrap()
            .filter_map(|value| match value {
                Ok(v) => { Some(v) }
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            });
        for row_data in cursor {
            let data = row_data.data;
            let collection_name = row_data.collection_name;
            let result = serde_json::from_str(data.as_str());
            let document = result.unwrap();
            if let Ok(_) = self.save_collection_inner(collection_name.as_str(), &document).await {
                match conn.execute(
                    "delete from saved where id=?1;",
                    [row_data.id],
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        num: i32,
    }

    use std::env;
    use mongodb::bson::doc;
    use crate::mongodb_saver::MongodbSaver;

    #[tokio::test]
    async fn test_sqlit() {
        let result = mongodb::bson::to_bson(&TestData { num: 1 }).unwrap();
        let now = chrono::Local::now();
        let document = doc! {"time":now, "data":&result};

        let saver_db_str = env::var("MongoDbSaverStr").expect("need saver db str");
        let mongodb_saver = MongodbSaver::init(saver_db_str.as_str()).await;
        mongodb_saver.write_local("aaa", &document).await;
        mongodb_saver.pop_local().await;
    }
}