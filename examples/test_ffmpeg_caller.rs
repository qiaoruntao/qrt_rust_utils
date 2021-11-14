use qrt_rust_utils::ffmpeg_caller::ffmpeg_caller::FFmpegCaller;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let source = "http://pull-hls-f26.douyincdn.com/stage/stream-109815816150319205.m3u8";
    let dest = "R:\\a.mp4";
    let args = vec!["-reconnect", "1", "-reconnect_at_eof", "1", "-reconnect_streamed", "1", "-reconnect_delay_max", "30", "-loglevel", "0", "-user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3844.0 Safari/537.36", "-i", source, "-c", "copy", dest];
    FFmpegCaller::new(args, ".").await;
}