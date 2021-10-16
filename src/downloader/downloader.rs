use std::path::Path;

use chrono::Local;

use crate::downloader::download_config::DownloadConfig;
use crate::downloader::download_info::DownloadInfo;

pub struct Downloader {}

impl Downloader {
    async fn download_file(url: &str, filepath: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = reqwest::get(url).await?;
        let mut file = std::fs::File::create(filepath)?;
        let mut content = std::io::Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;
        Ok(()) as Result<(), Box<dyn std::error::Error + Send + Sync>>
    }

    async fn download(download_info: &DownloadInfo, download_config: &DownloadConfig) {
        let url = download_info.download_url.clone();
        let download_directory = {
            let mut path = Path::new(&download_config.download_directory).to_path_buf();
            for directory in download_info.parent_directories.iter() {
                path = path.join(directory);
            }
            path
        };
        // let filename = Path::new(&url).file_name().unwrap().to_str().unwrap();

        let download_path = download_directory.join(&download_info.name);
        let download_path_str = download_path.as_os_str().to_str().unwrap();
        tokio::fs::create_dir_all(&download_directory).await;
        let updated_download_info = {
            let mut temp = download_info.clone();
            temp.start_time = Some(Local::now());
            temp.file_path = Some(download_path_str.into());
            temp.complete_time = Some(Local::now());
            temp
        };
        Downloader::download_file(url.as_str(), download_path_str).await;
    }
}