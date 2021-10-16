use chrono::Local;
use futures::TryStreamExt;
use mongodb::bson::Bson::Null;
use mongodb::bson::doc;
use mongodb::Collection;
use mongodb::options::InsertOneOptions;

use crate::mongodb_manager::mongodb_manager::MongoDbManager;
use crate::task::task::{Task, TaskMeta, TaskOptions, TaskState};

pub struct TaskScheduler {
    db_manager: MongoDbManager,
}

impl TaskScheduler {
    pub fn new(db_manager: MongoDbManager) -> TaskScheduler {
        TaskScheduler {
            db_manager
        }
    }

    pub async fn send_task(&self) -> Result<(), Box<dyn std::error::Error>> {
        let task = Task {
            key: "test".to_string(),
            meta: TaskMeta {
                name: "test".to_string(),
                create_time: Local::now(),
                creator: "default".to_string(),
            },
            option: TaskOptions::default(),
            task_state: TaskState {
                start_time: None,
                ping_time: None,
                next_ping_time: None,
                next_retry_time: None,
                complete_time: None,
                current_worker_id: None,
                progress: None,
            },
            param: 1,
            state: 1,
        };
        let mut options = mongodb::options::ReplaceOptions::default();
        options.upsert = Some(true);

        let collection = self.db_manager.get_collection::<Task<i32, i32>>("live_record");
        let result = collection.replace_one(task.generate_key_doc(), &task, Some(options)).await;
        dbg!(&result);
        println!("ok");
        Ok(())
    }

    // find tasks that we can process
    pub async fn find_pending_task(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut options = mongodb::options::FindOptions::default();
        let collection = self.db_manager.get_collection::<Task<i32, i32>>("live_record");

        let filter = doc! {
            "$and":[
                {"task_state.complete_time":Null},
                {
                    "$or":[
                        // not started
                        {"task_state.start_time":Null},
                        // started but not responding
                        {
                            "$and":[
                                {"task_state.next_ping_time":{"$ne":Null}},
                                {"task_state.next_ping_time":{"$lte":mongodb::bson::DateTime::now()}}
                            ]
                        }
                    ]
                }
            ]
        };
        dbg!(&filter);
        let mut cursor = collection.find(filter, Some(options)).await?;
        while let Some(doc) = cursor.try_next().await? {
            println!("{:#?}", doc);
        }
        println!("ok");
        Ok(())
    }
}

#[cfg(test)]
mod test_task_scheduler {
    use crate::config_manage::config_manager::ConfigManager;
    use crate::db_task::task_scheduler::TaskScheduler;
    use crate::mongodb_manager::mongodb_manager::MongoDbManager;

    #[tokio::test]
    async fn send_task() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        scheduler.send_task().await;
    }

    #[tokio::test]
    async fn find_pending_task() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        let result1 = scheduler.find_pending_task().await;
        dbg!(&result1);
    }
}