use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    url: String,
    #[serde(rename = "liveStreamId")]
    live_stream_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertWithTimestamp {
    timestamp: DateTime,
}
