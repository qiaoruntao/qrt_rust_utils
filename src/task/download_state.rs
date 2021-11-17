use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DownloadState {
    server_id: String,
    file_path: String,
}