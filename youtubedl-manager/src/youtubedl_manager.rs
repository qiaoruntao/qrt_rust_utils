use std::ffi::OsStr;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use async_recursion::async_recursion;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::io::{AsyncBufRead, AsyncRead};
use tokio::process::{ChildStdout, Command};
use tracing::{debug, error, info};
use tracing::field::debug;

pub struct YoutubeDlManager {}

#[async_recursion]
async fn check_progress<T: 'static + AsyncRead + Unpin + Send>(mut reader: Lines<BufReader<T>>, is_error: bool) -> Result<(), Box<dyn std::error::Error + Send>> {
    let result = &reader.next_line().await.unwrap();
    if let Some(str) = result {
        if is_error {
            error!("{}",str);
        } else {
            info!("{}",str);
        }
        tokio::spawn(check_progress(reader, is_error)).await.unwrap()
    } else {
        Ok(())
    }
}

impl YoutubeDlManager {
    pub async fn download(url: &str, path: &str) {
        let mut command = Command::new("youtube-dl");
        let command = command
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("--help");
        let mut child = command.spawn().unwrap();
        if let Some(output) = child.stdout.take() {
            let lines = BufReader::new(output).lines();
            tokio::spawn(check_progress(lines, false));
        }
        if let Some(output) = child.stderr.take() {
            let lines = BufReader::new(output).lines();
            tokio::spawn(check_progress(lines, true));
        }
        let x = child.wait().await;
        dbg!(x);
    }
}

mod test {
    use log_util::init_logger;

    use crate::youtubedl_manager::YoutubeDlManager;

    #[tokio::test]
    async fn test_download() {
        init_logger("test-youtube-dl", None);
        YoutubeDlManager::download("", "").await;
    }
}