use std::process::{Output, Stdio};

use cmd_lib::{CmdResult, run_fun};
use tokio::process::Command;
use tracing::error;

pub struct YoutubeDlUtils {}

impl YoutubeDlUtils {
    pub async fn download(
        filename: &str,
        download_directory: &str,
        url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let useragent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3844.0 Safari/537.36";
        if let Err(e) = tokio::fs::create_dir_all(download_directory).await {
            error!("cannot make download directory {}", download_directory);
            return Err(e.into());
        }

        let mut command =
            if cfg!(unix) {
                Command::new("python")
            } else {
                Command::new("youtube-dl")
            };

        let mut command =
            if cfg!(unix) {
                // TODO: how to fix this?
                let binary_path = "/home/qiaoruntao/.local/bin/youtube-dl";//run_fun!(which r"youtube-dl")?;
                if binary_path.is_empty() {
                    return Err("cannot find youtube-dl binary".into());
                }
                command.arg(binary_path)
            } else {
                &mut command
            };

        let command = command
            .stdin(Stdio::null())
            .kill_on_drop(true)
            .arg(url)
            .arg("--user-agent")
            .arg(useragent)
            // .arg("--hls-prefer-native")
            .arg("--socket-timeout")
            .arg("20")
            .arg("--hls-use-mpegts")
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
mod youtubedl_test {
    use std::time::Duration;

    use crate::youtubedl_utils::youtubedl_utils::YoutubeDlUtils;

    #[tokio::test(flavor = "current_thread")]
    async fn basic() {
        #[cfg(windows)]
            let directory = "R:/";
        #[cfg(unix)]
            let directory = "/mnt/r";
        let handle1 = tokio::spawn({
            let directory = directory.clone();
            async move {
                let result = YoutubeDlUtils::download("aaa", directory, "http://pull-hls-l11.douyincdn.com/stage/stream-109808439404265617.m3u8").await;
                dbg!(&result);
            }
        });
        let handle2 = tokio::spawn({
            let directory = directory.clone();
            async move {
                let result = YoutubeDlUtils::download("bbb", directory, "http://pull-hls-l11.douyincdn.com/stage/stream-109808439404265617.m3u8").await;
                dbg!(&result);
            }
        });
        tokio::join! {handle1, handle2}
        ;
    }
}
