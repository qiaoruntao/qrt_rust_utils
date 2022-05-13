use db_task::app::common::TaskAppBasicOperations;
use db_task::app::consumer::{TaskConsumeCore, TaskConsumeFunc, TaskConsumer};
use db_task::app::producer::TaskProducer;
use db_task::task::TaskConfig;
use db_task::task::TaskInfo;
use db_task::task::TaskRequest;
use db_task::tasker::single_tasker_producer::SingleTaskerProducer;

use crate::entity::tg_message::TgMessage;

pub mod entity;

#[derive(Debug)]
pub struct TgNotificationTaskInfo {}

impl TaskInfo for TgNotificationTaskInfo {
    type Params = TgMessage;
    type Returns = ();
}

pub struct NotificationManager {
    manager: SingleTaskerProducer<TgNotificationTaskInfo>,
}

impl NotificationManager {
    pub async fn send_text_message(&self, text: String) -> db_task::anyhow::Result<bool> {
        let tg_message = TgMessage::build_text(text);
        self.manager.send_new_task(tg_message).await
    }
    pub async fn init(connection_str: &str, collection_name: &str) -> NotificationManager {
        let notification_collection = SingleTaskerProducer::init(connection_str, collection_name).await;
        NotificationManager {
            manager: notification_collection
        }
    }
}