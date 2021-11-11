use std::collections::HashMap;
use std::sync::Arc;
use mongodb::{Client, Collection, Database};
use tokio::sync::RwLock;

pub struct MongoDbCollectionManager {
    pub client: Client,
    dbs: HashMap<String, Arc<RwLock<Database>>>,
    collections:HashMap<(String, String), Box<dyn Collection<T>>>
}

impl MongoDbCollectionManager {

}