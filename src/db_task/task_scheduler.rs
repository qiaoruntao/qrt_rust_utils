use std::fmt::Debug;

use chrono::Local;
use futures::TryStreamExt;
use mongodb::bson::Bson::Null;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::bson::oid::ObjectId;
use mongodb::Cursor;
use mongodb::error::{ErrorKind, WriteFailure};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::mongodb_manager::mongodb_manager::MongoDbManager;
use crate::task::task::{Task, TaskState};

#[derive(thiserror::Error, Debug)]
pub enum TaskSchedulerError {
    #[error("task exists")]
    TaskExists,
    #[error("no pending task exists")]
    NoPendingTask,
    #[error("no task matched")]
    NoMatchedTask,
    #[error("cannot occupy task")]
    OccupyTaskFailed,
    #[error("cannot complete task")]
    CompleteTaskFailed,
    #[error("cannot complete task")]
    CancelTaskFailed,
    #[error(transparent)]
    MongoDbError(#[from] mongodb::error::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct TaskScheduler {
    db_manager: MongoDbManager,
}

impl TaskScheduler {
    pub fn new(db_manager: MongoDbManager) -> TaskScheduler {
        TaskScheduler {
            db_manager
        }
    }

    pub async fn send_task<ParamType, StateType>(&self, task: &Task<ParamType, StateType>) -> Result<ObjectId, TaskSchedulerError>
        where ParamType: Debug + Serialize + DeserializeOwned + Unpin + Send + Sync, StateType: Debug + Serialize + DeserializeOwned + Unpin + Send + Sync {
        let options = mongodb::options::InsertOneOptions::default();

        let collection = self.db_manager.get_collection::<Task<ParamType, StateType>>("live_record");
        match collection.insert_one(task, options).await {
            Ok(insert_result) => {
                Ok(insert_result.inserted_id.as_object_id().unwrap())
            }
            Err(e) => {
                match e.kind.as_ref() {
                    ErrorKind::Write(WriteFailure::WriteError(write_error)) => {
                        if write_error.code == 11000 {
                            return Err(TaskSchedulerError::TaskExists);
                        }
                    }
                    _ => {}
                }
                Err(e.into())
            }
        }
    }

    // find tasks that we can process
    pub async fn find_next_pending_task<ParamType, StateType>(&self, custom_filter: Option<Document>) -> Result<Task<ParamType, StateType>, TaskSchedulerError>
        where ParamType: Debug + Serialize + DeserializeOwned + Unpin + Send + Sync, StateType: Debug + Serialize + DeserializeOwned + Unpin + Send + Sync, {
        let result = self.find_pending_task::<ParamType, StateType>(custom_filter).await;
        return match result {
            Err(e) => {
                Err(e)
            }
            Ok(mut cursor) => {
                let cursor_result = cursor.try_next().await?;
                match cursor_result {
                    Some(result) => {
                        Ok(result)
                    }
                    None => {
                        Err(TaskSchedulerError::NoPendingTask)
                    }
                }
            }
        };
    }

    // find tasks that we can process
    pub async fn find_all_pending_task<ParamType, StateType>(&self, custom_filter: Option<Document>) -> Result<Vec<Task<ParamType, StateType>>, TaskSchedulerError>
        where ParamType: Debug + Serialize + DeserializeOwned + Unpin + Send + Sync, StateType: Debug + Serialize + DeserializeOwned + Unpin + Send + Sync, {
        let result = self.find_pending_task::<ParamType, StateType>(custom_filter).await;
        return match result {
            Err(e) => {
                Err(e)
            }
            Ok(cursor) => {
                let cursor_result = cursor.try_collect::<Vec<Task<ParamType, StateType>>>().await;
                match cursor_result {
                    Ok(result) => {
                        Ok(result)
                    }
                    Err(e) => {
                        Err(e.into())
                    }
                }
            }
        };
    }

    // find tasks that we can process
    pub async fn find_pending_task<ParamType, StateType>(&self, custom_filter: Option<Document>) -> Result<Cursor<Task<ParamType, StateType>>, TaskSchedulerError>
        where ParamType: Debug + Serialize + DeserializeOwned + Unpin + Send + Sync, StateType: Debug + Serialize + DeserializeOwned + Unpin + Send + Sync, {
        let options = mongodb::options::FindOptions::default();
        let collection = self.db_manager.get_collection::<Task<ParamType, StateType>>("live_record");

        let filter = TaskScheduler::generate_pending_task_condition(custom_filter);
        // dbg!(&filter);
        let cursor = collection.find(filter, Some(options)).await?;
        // dbg!(&result);
        // println!("ok");
        Ok(cursor)
    }

    fn generate_pending_task_condition(custom_filter: Option<Document>) -> Document {
        let mut conditions = vec![
            doc! {"task_state.complete_time":Null},
            doc! {
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
                },
            doc! {"task_state.cancel_time":Null},
        ];
        if let Some(filter) = custom_filter {
            conditions.push(filter)
        }
        doc! {
            "$and":conditions
        }
    }

    // lock the task we want to handle
    pub async fn occupy_pending_task<ParamType, StateType>(&self, task: &Task<ParamType, StateType>) -> Result<(), TaskSchedulerError>
        where ParamType: Debug + Serialize + DeserializeOwned + Unpin, StateType: Debug + Serialize + DeserializeOwned + Unpin, {
        let collection = self.db_manager.get_collection::<Task<ParamType, StateType>>("live_record");

        let basic_filter = TaskScheduler::generate_pending_task_condition(None);
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

        // dbg!(&filter);
        // dbg!(&update);
        let mut options = mongodb::options::UpdateOptions::default();
        options.upsert = Some(false);

        let result = collection.update_one(filter, update, Some(options)).await?;
        dbg!(&result);
        if result.matched_count==0{
            TaskSchedulerError::NoMatchedTask
        }else if result.modified_count==0{
            TaskSchedulerError::CompleteTaskFailed
        }
        Ok(())
    }

    // mark task as completed
    pub async fn complete_task<ParamType, StateType>(&self, task: &Task<ParamType, StateType>) -> Result<(), TaskSchedulerError>
        where ParamType: Debug + Serialize + DeserializeOwned + Unpin, StateType: Debug + Serialize + DeserializeOwned + Unpin, {
        let collection = self.db_manager.get_collection::<Task<ParamType, StateType>>("live_record");

        // we need to make sure the task is being processed by ourself
        let filter = doc! {
            "task_state.complete_time": mongodb::bson::Bson::Null,
            "task_state.current_worker_id": &task.task_state.current_worker_id,
            "task_state.next_ping_time": { "$gte": mongodb::bson::DateTime::now() },
            "task_state.ping_time": { "$lte": mongodb::bson::DateTime::now() },
            "key":&task.key
        };
        let update = doc! {
            "$set":{
                "task_state.complete_time":mongodb::bson::DateTime::now(),
                "task_state.current_worker_id":mongodb::bson::Bson::Null,
                "task_state.next_ping_time": mongodb::bson::Bson::Null,
            }
        };

        dbg!(&filter);
        dbg!(&update);
        let mut options = mongodb::options::UpdateOptions::default();
        options.upsert = Some(false);

        let result = collection.update_one(filter, update, Some(options)).await?;
        dbg!(&result);
        if result.matched_count == 0 {
            Err(TaskSchedulerError::NoMatchedTask)
        } else if result.modified_count == 0 {
            Err(TaskSchedulerError::CompleteTaskFailed)
        } else {
            Ok(())
        }
    }

    // mark task as cancelled
    pub async fn cancel_task<ParamType, StateType>(&self, task: &Task<ParamType, StateType>) -> Result<(), TaskSchedulerError>
        where ParamType: Debug + Serialize + DeserializeOwned + Unpin, StateType: Debug + Serialize + DeserializeOwned + Unpin, {
        let collection = self.db_manager.get_collection::<Task<ParamType, StateType>>("live_record");

        // we need to make sure the task is being processed by ourself
        let filter = doc! {
            "task_state.complete_time": mongodb::bson::Bson::Null,
            "task_state.cancel_time": mongodb::bson::Bson::Null,
            // "task_state.current_worker_id": &task.task_state.current_worker_id,
            "key":&task.key
        };
        let update = doc! {
            "$set":{
                "task_state.cancel_time":mongodb::bson::DateTime::now(),
                "task_state.current_worker_id":mongodb::bson::Bson::Null,
                "task_state.next_ping_time": mongodb::bson::Bson::Null,
            }
        };

        dbg!(&filter);
        dbg!(&update);
        let mut options = mongodb::options::UpdateOptions::default();
        options.upsert = Some(false);

        let result = collection.update_one(filter, update, Some(options)).await?;
        dbg!(&result);
        if result.matched_count == 0 {
            Err(TaskSchedulerError::NoMatchedTask)
        } else if result.modified_count == 0 && result.upserted_id == None {
            Err(TaskSchedulerError::CancelTaskFailed)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test_task_scheduler {
    use chrono::Local;
    use futures::TryStreamExt;
    use tracing::error;

    use crate::config_manage::config_manager::ConfigManager;
    use crate::db_task::task_scheduler::TaskScheduler;
    use crate::mongodb_manager::mongodb_manager::MongoDbManager;
    use crate::task::task::{Task, TaskMeta, TaskOptions, TaskState};

    #[tokio::test]
    async fn send_task() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
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
                cancel_time: None,
                current_worker_id: None,
                progress: None,
            },
            param: 1,
            state: 1,
        };
        let send_result = scheduler.send_task(&task).await;
        dbg!(&send_result);
    }

    #[tokio::test]
    async fn find_pending_task() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        let result1 = scheduler.find_pending_task::<i32, i32>(None).await.unwrap();
        dbg!(&result1);
    }

    #[tokio::test]
    async fn complete_pending_task() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        let mut all_pending_tasks = scheduler.find_pending_task::<i32, i32>(None).await.unwrap();

        let task = match all_pending_tasks.try_next().await {
            Err(e) => {
                error!("{:?}", &e);
                return;
            }
            Ok(task) => task.unwrap(),
        };
        let occupy_result = scheduler.occupy_pending_task(&task).await;
        dbg!(&occupy_result);
        let complete_result = scheduler.complete_task(&task).await;
        dbg!(&complete_result);
    }

    #[tokio::test]
    async fn cancel_pending_task() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        let mut all_pending_tasks = scheduler.find_pending_task::<i32, i32>(None).await.unwrap();

        let task = match all_pending_tasks.try_next().await {
            Err(e) => {
                error!("{:?}", &e);
                return;
            }
            Ok(task) => task.unwrap(),
        };

        let occupy_result = scheduler.occupy_pending_task(&task).await;
        dbg!(&occupy_result);
        let complete_result = scheduler.cancel_task(&task).await;
        dbg!(&complete_result);
    }
}