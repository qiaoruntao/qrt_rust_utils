use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct DownloadConfig {
    pub download_directory: String,
}
