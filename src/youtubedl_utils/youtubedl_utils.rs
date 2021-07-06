use std::process::Stdio;

use tokio::process::Command;


pub struct YoutubeDlUtils {}

impl YoutubeDlUtils {
    async fn download(filename: &str, download_directory: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new("youtube-dl");
        let command = command
            .stdin(Stdio::null())
            // .arg(format!("--referer={referer}", referer = document.url))
            .arg(url)
            // .envs(x1)
            .arg("--hls-prefer-native")
            .arg("-o")
            .arg(format!("{title}.%(ext)s", title = filename))
            .current_dir(download_directory);
        // start download
        let mut child = match command.spawn() {
            Ok(thread) => thread,
            Err(e) => {
                panic!("failed to spawn youtube-dl, {}", e);
            }
        };
        // Await until the command completes
        let status = match child.wait().await {
            Ok(result) => result,
            Err(e) => {
                println!("youtube-dl failed, {}", e);
                return Err(e.into());
            }
        };
        if let Some(status_code) = status.code() {
            if status_code != 0 {
                return Err(format!("failed to download file {}", filename).into());
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod youtubedl_test{
    use crate::youtubedl_utils::youtubedl_utils::YoutubeDlUtils;

    #[tokio::test]
    async fn basic(){
        let _result = YoutubeDlUtils::download("aaa", "R:/", "https://www.youtube.com/watch?v=MOnsIGcZS-M").await;
    }
}