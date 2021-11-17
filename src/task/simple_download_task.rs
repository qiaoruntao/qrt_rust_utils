use serde::{Deserialize, Serialize};

use crate::task::download_state::DownloadState;
use crate::task::task::Task;

// a very basic get download
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SimpleDownloadTaskParam {
    // for filename generation
    pub name: String,
    // generate the directory to store the file
    pub parent_directories: Vec<String>,
    // which link to download
    pub download_url: String,
}

pub type SimpleDownloadTask = Task<SimpleDownloadTaskParam, DownloadState>;