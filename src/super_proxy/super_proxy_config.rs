use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct SuperProxyConfig {
    pub address: String,
    pub username: String,
    pub password: String,
}