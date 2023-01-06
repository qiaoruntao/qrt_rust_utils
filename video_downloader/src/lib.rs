use anyhow::anyhow;

use ffmpeg_caller::ffmpeg_caller::FfmpegCaller;
use log_util::tracing::instrument;

pub struct VideoDownloader {}

impl VideoDownloader {
    #[instrument]
    pub async fn download(url: &str, path: &str) -> anyhow::Result<()> {
        let args = vec![
            // some site have strange protocol, like hist
            "-protocol_whitelist", "file,http,https,tcp,tls,hist",
            // "-loglevel", "0",
            "-user_agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3844.0 Safari/537.36",
            "-i", url,
            "-f", "mpegts",
            "-c", "copy",
            "-reconnect", "1",
            "-reconnect_at_eof", "1",
            "-reconnect_streamed", "1",
            "-reconnect_delay_max", "2",
            path,
        ];
        let result = FfmpegCaller::run(args, ".").await;
        match result {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(anyhow!(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::VideoDownloader;

    #[tokio::test]
    async fn it_works() {
        VideoDownloader::download("http://pull-hls-l1.douyincdn.com/radio/stream-110970056259928098/playlist.m3u8", "R:\\test.mp3").await;
    }
}
