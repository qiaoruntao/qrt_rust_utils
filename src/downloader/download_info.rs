use chrono::{DateTime, Local};
use derive_builder::*;
use serde::{Deserialize, Serialize};

/// record what and when you downloaded
#[derive(Deserialize, Serialize, Debug, Builder, Clone)]
pub struct DownloadInfo {
    // set by download initiator

    // for filename generation
    pub name: String,
    // generate the directory to store the file
    pub parent_directories: Vec<String>,
    // which link to download
    pub download_url: String,
    // TODO: reqwest params

    // set by downloader
    pub start_time: Option<DateTime<Local>>,
    pub file_path: Option<String>,
    pub complete_time: Option<DateTime<Local>>,
    // how many times have we retried
    pub retry_cnt: u32,
}
