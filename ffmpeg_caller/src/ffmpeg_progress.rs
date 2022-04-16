use std::time::Duration;

/// A progress event emitted by ffmpeg.
///
/// Names of the fields directly correspond to the names in the output of ffmpeg's `-progress`.
/// Everything is wrapped in an option because this has no docs I can find, so I can't guarantee
/// that they will all be in the data ffmpeg sends.
/// Note that bitrate is ignored because I'm not sure of the exact format it's in. Blame ffmpeg.
#[derive(Debug, Default)]
pub struct Progress {
    /// What frame ffmpeg is on.
    pub frame: Option<u64>,
    /// What framerate ffmpeg is processing at.
    pub fps: Option<f64>,
    /// How much data ffmpeg has output so far, in bytes.
    pub total_size: Option<u64>,
    /// How far ffmpeg has processed.
    pub out_time: Option<Duration>,
    /// How many frames were duplicated? The meaning is unclear.
    pub dup_frames: Option<u64>,
    /// How many frames were dropped.
    pub drop_frames: Option<u64>,
    /// How fast it is processing, relative to 1x playback speed.
    pub speed: Option<f64>,
    /// What ffmpeg will do now.
    pub status: Status,
}

/// What ffmpeg is going to do next.
#[derive(Debug)]
pub enum Status {
    /// Ffmpeg will continue emitting progress events.
    Continue,
    /// Ffmpeg has finished processing.
    ///
    /// After emitting this, the stream will end.
    End,
}

impl Default for Status {
    fn default() -> Self {
        Status::Continue
    }
}