use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::task::task::{deserialize_datetime_as_datetime, serialize_datetime_as_datetime};

#[derive(Deserialize, Serialize, Debug)]
pub struct SavedRecord<T> {
    #[serde(
    serialize_with = "serialize_datetime_as_datetime",
    deserialize_with = "deserialize_datetime_as_datetime"
    )]
    pub time: DateTime<Local>,
    pub data: T,
}