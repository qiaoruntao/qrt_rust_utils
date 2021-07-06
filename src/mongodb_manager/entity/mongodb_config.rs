use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct MongoDbConfig {
    pub address: String,
    pub username: String,
    pub password: String,
    pub port: u16,
    pub source: String,
    pub use_tls: bool,
}
