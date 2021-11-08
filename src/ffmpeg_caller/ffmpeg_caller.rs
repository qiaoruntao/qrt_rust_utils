use std::ffi::OsStr;
use std::io::Stdout;
use std::process::Stdio;
use std::sync::mpsc;

use async_recursion::async_recursion;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{ChildStdout, Command};

pub struct FFmpegCaller {}

#[async_recursion]
async fn check_progress(mut reader: Lines<BufReader<ChildStdout>>) -> Result<(), Box<dyn std::error::Error + Send>> {
    let result = &reader.next_line().await.unwrap();
    if let Some(str) = result {
        println!("Line1: {}", str);
        tokio::spawn(check_progress(reader)).await.unwrap()
    } else {
        Ok(())
    }
}

fn parse_line(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    let mut iter = trimmed.splitn(2, '=');

    let mut key = iter.next()?;
    key = key.trim_end();

    let mut value = iter.next()?;
    // Ffmpeg was putting in random spaces for some reason?
    value = value.trim_start();

    Some((key, value))
}

impl FFmpegCaller {
    pub async fn new<I, S>(args: I, working_directory: &str)
        where
            I: IntoIterator<Item=S>,
            S: AsRef<OsStr>, {
        let mut command = Command::new("ffmpeg");
        let command = command
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .args(args)
            .arg("-progress")
            .arg("-nostats")
            .current_dir(working_directory);
        let mut child = command.spawn().unwrap();
        let output = match child.stdout.take() {
            None => {
                panic!("no output");
            }
            Some(e) => {
                e
            }
        };
        let reader = BufReader::new(output).lines();
        let result = check_progress(reader).await;
        dbg!(&result);
    }
}

#[cfg(test)]
mod test_ffmpeg_caller {
    use crate::ffmpeg_caller::ffmpeg_caller::FFmpegCaller;

    #[tokio::test]
    async fn test() {
        FFmpegCaller::new(vec!["--help"], ".").await;
    }
}