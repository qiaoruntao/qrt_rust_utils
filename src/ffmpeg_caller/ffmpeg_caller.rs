use std::ffi::OsStr;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use async_recursion::async_recursion;
use regex::Regex;
use tokio::io::{BufReader, Lines};
use tokio::process::{ChildStdout, Command};
use tracing::{error, trace};

use crate::ffmpeg_caller::ffmpeg_progress::{Progress, Status};

pub struct FFmpegCaller {}

#[async_recursion]
async fn check_progress(mut reader: Lines<BufReader<ChildStdout>>) -> Result<(), Box<dyn std::error::Error + Send>> {
    let result = &reader.next_line().await.unwrap();
    if let Some(str) = result {
        let progress = parse_line(str);
        trace!("&progress={:?}",&progress);
        tokio::spawn(check_progress(reader)).await.unwrap()
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

impl FFmpegCaller {
    pub async fn run<I, S>(args: I, working_directory: &str) -> tokio::io::Result<ExitStatus>
        where
            I: IntoIterator<Item=S>,
            S: AsRef<OsStr>, {
        let mut command = Command::new("ffmpeg");
        let command = command
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .arg("-stats")
            .arg("-y")
            .arg("-loglevel")
            .arg("quiet")
            .args(args)
            .current_dir(working_directory);
        let mut child = command.spawn().unwrap();
        // let output = match child.stdout.take() {
        //     None => {
        //         panic!("no output");
        //     }
        //     Some(e) => {
        //         e
        //     }
        // };
        // let reader = BufReader::new(output).lines();
        let result = child.wait().await;
        trace!("&result={:?}",&result);
        result
    }
}

#[cfg(test)]
mod test_ffmpeg_caller {
    use tracing::trace;
    use crate::ffmpeg_caller::ffmpeg_caller::{FFmpegCaller, parse_line};

    #[tokio::test]
    async fn test() {
        let source = "R:\\a.mp4";
        let dest = "R:\\a.mkv";
        let args = vec!["-reconnect", "1", "-reconnect_at_eof", "1", "-reconnect_streamed", "1", "-reconnect_delay_max", "30", "-loglevel", "0", "-user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3844.0 Safari/537.36", "-i", source, "-c", "copy", dest];
        FFmpegCaller::run(args, ".").await;
    }

    #[test]
    fn test_parse_line() {
        let line = "frame=  196 fps= 19 q=-1.0 Lsize=    5081kB time=00:00:14.06 bitrate=2958.7kbits/s dup=2 drop=0 speed=1.36x";
        let progress = parse_line(line);
        trace!("&progress={:?}",&progress);
    }
}