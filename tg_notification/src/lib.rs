use std::time::{SystemTime, UNIX_EPOCH};
use mscheduler::tasker::error::MResult;
use mscheduler::tasker::producer::{SendTaskResult, TaskProducer};
use mscheduler::util::get_collection;

use crate::entity::tg_message::TgMessage;

pub mod entity;

pub struct NotificationManager {
    manager: TaskProducer<TgMessage, ()>,
}

impl NotificationManager {
    pub async fn send_text_message(&self, text: String) -> MResult<SendTaskResult> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = current_time.as_secs();
        let key = timestamp.to_string();
        let tg_message = TgMessage::build_text(text);
        self.manager.send_task(key, tg_message, None).await
    }
    pub async fn init(connection_str: impl AsRef<str>, collection_name: impl AsRef<str>) -> NotificationManager {
        let collection = get_collection(connection_str, collection_name).await;
        let manager = TaskProducer::create(collection).expect("failed to init NotificationManager");
        NotificationManager {
            manager
        }
    }
}