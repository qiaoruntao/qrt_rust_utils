use std::process::Stdio;

use anyhow::anyhow;
use async_recursion::async_recursion;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::io::AsyncRead;
use tokio::process::Command;
use tracing::{debug, error, info};

pub struct YoutubeDlManager {}

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[async_recursion]
async fn check_progress<T: 'static + AsyncRead + Unpin + Send>(mut reader: Lines<BufReader<T>>, is_error: bool) -> Result<(), Box<dyn std::error::Error + Send>> {
    let result = &reader.next_line().await.unwrap();
    if let Some(str) = result {
        if is_error {
            error!("{}",str);
        } else {
            info!("{}",str);
        }
        // TODO: best practise?
        tokio::spawn(check_progress(reader, is_error)).await.unwrap()
    } else {
        Ok(())
    }
}

impl YoutubeDlManager {
    // TODO: 返回值?
    pub async fn download(url: &str, path: &str) -> anyhow::Result<()> {
        let mut command = Command::new("youtube-dl");
        let useragent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3844.0 Safari/537.36";
        let mut command = command
            .stdin(Stdio::null())
            // make sure we won't retry infinitely
            .arg("--skip-unavailable-fragments")
            .arg("--retries")
            .arg("10")
            .arg("--fragment-retries")
            .arg("10")
            // broadcasting
            .arg("--hls-use-mpegts")
            .arg("--user-agent")
            .arg(useragent)
            .arg("--format")
            .arg("best")
            .arg("--no-part")
            .arg(url)
            .arg("-o")
            // sanitize template cause all parameters can be generated previously
            .arg(path);
        if cfg!(debug_assertions) {
            command = command
                .stderr(Stdio::piped())
                .stdout(Stdio::piped());
        } else {
            command = command
                .stderr(Stdio::null())
                .stdout(Stdio::null());
        }
        // do not show console window for every youtube-dl process
        if cfg!(windows) {
            command = command.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = match command.spawn() {
            Ok(child) => {
                child
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        if cfg!(debug_assertions) {
            if let Some(output) = child.stdout.take() {
                let lines = BufReader::new(output).lines();
                tokio::spawn(check_progress(lines, false));
            }
            if let Some(output) = child.stderr.take() {
                let lines = BufReader::new(output).lines();
                tokio::spawn(check_progress(lines, true));
            }
        }
        let x = child.wait().await;
        match x {
            Ok(value) => {
                debug!("{:?}", value);
                Ok(())
            }
            Err(e) => {
                // 似乎没遇到过
                Err(anyhow!(e))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use log_util::init_logger;

    use crate::youtubedl_manager::YoutubeDlManager;

    //noinspection HttpUrlsUsage
    #[tokio::test]
    async fn test_download() {
        init_logger("test-youtube-dl", None);
        YoutubeDlManager::download("http://pull-hls-l26.douyincdn.com/stage/stream-110735334551584907.m3u8", "D:\\test%(timestamp)s.mp4").await.unwrap();
    }
}