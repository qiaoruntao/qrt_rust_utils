use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyPoolGetResponse {
    pub anonymous: String,
    pub check_count: i64,
    pub fail_count: i64,
    pub https: bool,
    pub last_status: bool,
    pub last_time: String,
    pub proxy: String,
    pub region: String,
    pub source: String,
}
