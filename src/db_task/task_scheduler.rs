use chrono::Local;
use futures::TryStreamExt;
use mongodb::bson::Bson::Null;
use mongodb::bson::doc;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

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
    pub async fn find_pending_task(&self) -> Result<Box<Vec<Task<i32, i32>>>, Box<dyn std::error::Error>> {
        let options = mongodb::options::FindOptions::default();
        let collection = self.db_manager.get_collection::<Task<i32, i32>>("live_record");

        let filter = TaskScheduler::generate_pending_task_condition();
        // dbg!(&filter);
        let cursor = collection.find(filter, Some(options)).await?;
        let result = cursor.try_collect::<Vec<_>>().await?;
        dbg!(&result);
        // println!("ok");
        Ok(Box::from(result))
    }

    fn generate_pending_task_condition() -> Document {
        doc! {
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
        }
    }

    // lock the task we want to handle
    pub async fn occupy_pending_task<ParamType, StateType>(&self, task: &Task<ParamType, StateType>) -> Result<(), Box<dyn std::error::Error>>
        where ParamType: Debug + Serialize + DeserializeOwned + Unpin, StateType: Debug + Serialize + DeserializeOwned + Unpin, {
        let collection = self.db_manager.get_collection::<Task<ParamType, StateType>>("live_record");

        let basic_filter = TaskScheduler::generate_pending_task_condition();
        // we need to make sure the task is still pending
        let filter = doc! {
            "$and":[
                basic_filter,
                {"key":&task.key}
            ]
        };
        let new_state = TaskState::init(&Local::now(), 1, &task.option);
        let update = doc! {
            "$set":{
                "task_state":mongodb::bson::to_document( &new_state).unwrap()
            }
        };

        dbg!(&filter);
        dbg!(&update);
        let mut options = mongodb::options::UpdateOptions::default();
        options.upsert = Some(false);

        let result = collection.update_one(filter, update, Some(options)).await?;
        dbg!(&result);
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
        scheduler.send_task().await.unwrap();
    }

    #[tokio::test]
    async fn find_pending_task() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        let result1 = scheduler.find_pending_task().await.unwrap();
        dbg!(&result1);
    }

    #[tokio::test]
    async fn occupy_pending_task() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        let result1 = scheduler.find_pending_task().await.unwrap();

        let task = &result1[0];
        let result2 = scheduler.occupy_pending_task(task).await;
        dbg!(&result2);
    }
}