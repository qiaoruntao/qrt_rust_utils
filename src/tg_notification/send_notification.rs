use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::error;

use crate::db_task::task_scheduler::TaskScheduler;
use crate::mongodb_manager::mongodb_manager::MongoDbManager;
use crate::task::task::{DefaultTaskState, Task, TaskMeta, TaskState};
use crate::tg_notification::entity::tg_message::TgMessage;

pub struct NotificationManager {
    task_scheduler: Arc<RwLock<TaskScheduler>>,
}

impl NotificationManager {
    pub fn new(db_manager: MongoDbManager) -> NotificationManager {
        let task_scheduler = TaskScheduler::new(db_manager, "notifications".into());
        NotificationManager {
            task_scheduler: Arc::new(RwLock::new(task_scheduler)),
        }
    }
    pub fn send_notification(&self, msg: TgMessage) {
        let task_options = Default::default();
        let task_state = TaskState::from(&task_options);
        let task = Task {
            key: msg.gen_hash().to_string(),
            meta: TaskMeta::create_now("Telegram Message".into(), "utils".into()),
            option: task_options,
            task_state,
            param: msg,
            state: DefaultTaskState::default(),
        };
        let arc = self.task_scheduler.clone();
        // async send, do not block
        tokio::spawn(async move {
            let guard = arc.try_read().unwrap();
            match guard.send_task(Arc::new(RwLock::new(task))).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{:?}", &e);
                }
            }
        });
    }
}

#[cfg(test)]
mod notification_test {
    use std::time::Duration;

    use crate::config_manage::config_manager::ConfigManager;
    use crate::mongodb_manager::mongodb_manager::MongoDbManager;
    use crate::tg_notification::entity::tg_message::TgMessage;
    use crate::tg_notification::send_notification::NotificationManager;

    #[tokio::test]
    async fn basic() {
        let mongodb_config = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(mongodb_config, "Logger").unwrap();
        let notification_manager = NotificationManager::new(db_manager);
        let tg_message = TgMessage::build_text("test".into());
        notification_manager.send_notification(tg_message);
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
