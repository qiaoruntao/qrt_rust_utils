use std::ffi::OsStr;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use async_recursion::async_recursion;
use regex::Regex;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::io::AsyncRead;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::ffmpeg_progress::{Progress, Status};

pub struct FfmpegCaller {}

#[async_recursion]
async fn check_progress<T: 'static + AsyncRead + Unpin + Send>(mut reader: Lines<BufReader<T>>, is_error: bool) -> Result<(), Box<dyn std::error::Error + Send>> {
    let result = &reader.next_line().await.unwrap();
    if let Some(str) = result {
        let progress = parse_line(str);
        if is_error {
            error!("&progress={:?}",&progress);
        } else {
            info!("&progress={:?}",&progress);
        }
        tokio::spawn(check_progress(reader, is_error)).await.unwrap()
    } else {
        Ok(())
    }
}

fn parse_line(line: &str) -> Progress {
    let re = Regex::new(r"(?P<key>\w+)=\s*(?P<value>\S+)").unwrap();
    let mut progress = Progress::default();
    for captures in re.captures_iter(line).into_iter() {
        let key = &captures["key"];
        let value = &captures["value"];
        match key {
            "frame" => match value.parse() {
                Ok(x) => progress.frame = Some(x),
                Err(e) => error!("{}", e),
            },
            "fps" => match value.parse() {
                Ok(x) => progress.fps = Some(x),
                Err(e) => error!("{}", e),
            },
            "total_size" => match value.parse() {
                Ok(x) => progress.total_size = Some(x),
                Err(e) => error!("{}", e),
            },
            "out_time_us" => match value.parse() {
                Ok(us) => progress.out_time = Some(Duration::from_micros(us)),
                Err(e) => error!("{}", e),
            },
            "dup" | "dup_frames" => match value.parse() {
                Ok(x) => progress.dup_frames = Some(x),
                Err(e) => error!("{}", e),
            },
            "drop" | "drop_frames" => match value.parse() {
                Ok(x) => progress.drop_frames = Some(x),
                Err(e) => error!("{}", e),
            },
            "speed" => {
                let num = &value[..(value.len() - 1)];
                match num.parse() {
                    Ok(x) => progress.speed = Some(x),
                    Err(e) => error!("{}", e),
                }
            }
            "progress" => {
                progress.status = match value {
                    "continue" => Status::Continue,
                    "end" => Status::End,
                    _ => {
                        // Just give it a status so it compiles
                        Status::End
                    }
                };
            }
            &_ => {}
        }
    }
    progress
}

impl FfmpegCaller {
    pub async fn run<I, S>(args: I, working_directory: &str) -> tokio::io::Result<ExitStatus>
        where
            I: IntoIterator<Item=S>,
            S: AsRef<OsStr>, {
        let mut command = Command::new("ffmpeg");
        let command = command
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .arg("-stats")
            .arg("-y")
            .arg("-loglevel")
            .arg("quiet")
            .args(args)
            .current_dir(working_directory);
        dbg!(&command);
        let mut child = command.spawn().unwrap();
        if let Some(output) = child.stdout.take() {
            let lines = BufReader::new(output).lines();
            tokio::spawn(check_progress(lines, false));
        }
        if let Some(output) = child.stderr.take() {
            let lines = BufReader::new(output).lines();
            tokio::spawn(check_progress(lines, true));
        }
        debug!("command spawned");
        let status = child.wait().await;
        debug!("status is {:?}", &status);
        status
    }
}

#[cfg(test)]
mod test_ffmpeg_caller {
    use tracing::trace;

    use crate::ffmpeg_caller::{FfmpegCaller, parse_line};

    #[tokio::test]
    async fn test() {
        let source = "R:\\a.mp4";
        let dest = "R:\\a.mkv";
        let args = vec!["-reconnect", "1", "-reconnect_at_eof", "1", "-reconnect_streamed", "1", "-reconnect_delay_max", "30", "-loglevel", "0", "-user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3844.0 Safari/537.36", "-i", source, "-c", "copy", dest];
        dbg!(&FfmpegCaller::run(args, ".").await);
    }

    #[test]
    fn test_parse_line() {
        let line = "frame=  196 fps= 19 q=-1.0 Lsize=    5081kB time=00:00:14.06 bitrate=2958.7kbits/s dup=2 drop=0 speed=1.36x";
        let progress = parse_line(line);
        trace!("&progress={:?}",&progress);
    }
}