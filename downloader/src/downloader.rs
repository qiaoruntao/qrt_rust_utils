use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use url::Url;

use sanitizer::sanitize::sanitize;
use crate::download_config::DownloadConfig;

pub struct Downloader {}

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

// this module is under development
impl Downloader {
    #[instrument]
    pub async fn download_file(
        url: &str,
        filepath: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = reqwest::get(url).await?;
        let mut file = std::fs::File::create(filepath)?;
        let mut content = std::io::Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;
        Ok(()) as Result<(), Box<dyn std::error::Error + Send + Sync>>
    }

    #[instrument]
    pub async fn download(download_info: &SimpleDownloadTaskParam, download_config: &DownloadConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = download_info.download_url.clone();
        let download_directory = {
            let mut path = Path::new(&download_config.download_directory).to_path_buf();
            for directory in download_info.parent_directories.iter() {
                let str = sanitize(directory);
                let str = if str.len() > 250 {
                    str.chars().take(125).collect::<String>()
                } else {
                    str
                };
                path = path.join(str);
            }
            path
        };
        let parsed_url = Url::parse(url.as_str()).unwrap();
        let all_split = parsed_url.path_segments().unwrap();
        let _last_part = all_split.last().unwrap();
        // let filename = match last_part.split('.').last() {
        //     None => {
        //         info!("extension info not found, url={}", url);
        //         // no extension info, use name only
        //         download_info.name.clone()
        //     }
        //     Some(ext) => {
        //         format!("{}.{}", download_info.name.as_str(), ext)
        //     }
        // };

        let filename = sanitize(&download_info.name);
        let filename = if filename.len() > 250 {
            filename.chars().take(125).collect::<String>()
        } else {
            filename
        };
        let download_path = download_directory.join(filename);
        let download_path_str = download_path.as_os_str().to_str().unwrap();
        match tokio::fs::create_dir_all(&download_directory).await {
            Ok(_) => {}
            Err(err) => {
                error!("create_dir_all failed {} {:?}",download_path_str, &err);
                return Err(err.into());
            }
        }
        Downloader::download_file(url.as_str(), download_path_str)
            .await
    }
}
