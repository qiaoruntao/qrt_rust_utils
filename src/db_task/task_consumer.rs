use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mongodb::bson::Bson::ObjectId;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Instant, interval_at};
use tracing::{error, info};

use crate::db_task::task_scheduler::{TaskScheduler, TaskSchedulerError};
use crate::task::task::Task;

pub enum TaskConsumerResult {
    Completed,
    Cancelled,
    Failed,
}

#[async_trait]
pub trait TaskConsumer<ParamType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send, StateType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send> {
    fn build_filter(&self) -> Option<Document>;
    // define how to consume a task
    async fn consume(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> TaskConsumerResult;
    // used to run background maintain
    async fn add_running_task(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> bool;
    async fn remove_running_task(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> bool;
    // maintain task
    async fn run_maintainer(&self, task_scheduler: Arc<RwLock<TaskScheduler>>);

    async fn run_task_core(&self, task_scheduler: Arc<RwLock<TaskScheduler>>, task: Arc<RwLock<Task<ParamType, StateType>>>) -> Result<(), TaskSchedulerError> {
        let task_scheduler_guard = task_scheduler.try_read().unwrap();
        // background maintainer
        let key = &task.try_read().unwrap().key;
        let is_added = self.add_running_task(task.clone()).await;
        if !is_added {
            error!("task {} already running", &key);
            return Err(TaskSchedulerError::MaintainerError);
        }
        // check if we are running maintainer

        // run the task
        let consumer_result = self.consume(task.clone());
        // remove maintainer
        let is_removed = self.remove_running_task(task.clone()).await;
        if !is_removed {
            error!("task {} not exists", &key);
            return Err(TaskSchedulerError::MaintainerError);
        }
        // update result
        match consumer_result.await {
            TaskConsumerResult::Completed => {
                task_scheduler_guard.complete_task(task.clone()).await
            }
            TaskConsumerResult::Cancelled => {
                task_scheduler_guard.cancel_task(task.clone()).await
            }
            TaskConsumerResult::Failed => {
                // TODO: do nothing, wait for timeout retry
                Err(TaskSchedulerError::TaskFailedError)
            }
        }
    }
}

pub struct WebcastConsumer {
    running_tasks: Mutex<Vec<Arc<RwLock<Task<i32, i32>>>>>,
}

#[async_trait]
impl TaskConsumer<i32, i32> for WebcastConsumer {
    fn build_filter(&self) -> Option<Document> {
        None
    }

    async fn consume(&self, task: Arc<RwLock<Task<i32, i32>>>) -> TaskConsumerResult {
        info!("task {} consumed", &task.try_read().unwrap().key);

        TaskConsumerResult::Completed
    }

    async fn add_running_task(&self, task: Arc<RwLock<Task<i32, i32>>>) -> bool {
        let mut guard = self.running_tasks.lock().await;
        guard.borrow_mut().push(task);
        true
    }

    async fn remove_running_task(&self, task2remove: Arc<RwLock<Task<i32, i32>>>) -> bool {
        let mut guard = self.running_tasks.lock().await;
        for (index, task) in guard.iter().enumerate() {
            if task.try_read().unwrap().deref() == task2remove.try_read().unwrap().deref() {
                guard.swap_remove(index);
                return true;
            }
        }
        false
    }

    async fn run_maintainer(&self, task_scheduler: Arc<RwLock<TaskScheduler>>) {
        let guard = self.running_tasks.lock().await;
        info!("maintainer runs, task count= {}", guard.len());
        let scheduler_guard = task_scheduler.try_read().unwrap();
        for running_task in guard.iter() {
            let update_result = scheduler_guard.update_task(running_task.clone()).await;
            match update_result {
                Ok(_) => {}
                Err(e) => {
                    dbg!("{:?}",e);
                }
            }
        }
    }
}


#[cfg(test)]
mod test_task_scheduler {
    use std::sync::Arc;

    use futures::future::err;
    use tokio::sync::RwLock;
    use tokio::time::Duration;
    use tracing::error;
    use tracing::info;

    use crate::config_manage::config_manager::ConfigManager;
    use crate::db_task::task_consumer::{TaskConsumer, WebcastConsumer};
    use crate::db_task::task_scheduler::{TaskScheduler, TaskSchedulerError};
    use crate::logger::logger::Logger;
    use crate::mongodb_manager::mongodb_manager::MongoDbManager;

    #[tokio::test]
    async fn test() {
        let logger_config = ConfigManager::read_config_with_directory("./config/logger").unwrap();
        Logger::init_logger(&logger_config);
        let mongodb_config = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(mongodb_config, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        let webcast_consumer = WebcastConsumer {
            running_tasks: Default::default()
        };
        let arc_scheduler = Arc::new(RwLock::new(scheduler));
        let arc_consumer = Arc::new(RwLock::new(webcast_consumer));
        loop {
            let arc1 = arc_consumer.clone();
            let consumer = arc1.try_read().unwrap();
            let filter = consumer.build_filter();
            // maintain
            let arc2 = arc_scheduler.clone();
            consumer.run_maintainer(arc2.clone()).await;
            drop(consumer);
            // find a task
            let task_scheduler_guard = arc2.try_read().unwrap();
            // TODO: merge find and occupy for relentless check
            let result = task_scheduler_guard.find_next_pending_task(filter.clone()).await;
            let task = match result {
                Ok(task) => task,
                Err(e) => {
                    match e {
                        TaskSchedulerError::NoPendingTask => {
                            info!("no pending task found");
                            // wait a while
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                            // break;
                        }
                        _ => {
                            // unexpected error
                            error!("{:?}", e);
                            // break;
                            break;
                        }
                    }
                }
            };
            info!("new task found");
            // occupy the task
            if let Err(e) = task_scheduler_guard.occupy_pending_task(task.clone()).await {
                match e {
                    TaskSchedulerError::OccupyTaskFailed | TaskSchedulerError::NoMatchedTask => {
                        // this is ok
                        info!("occupy task failed, {:?}", e);
                    }
                    _ => {
                        error!("unknown error while occupying task {:?}",e);
                    }
                }
            }
            drop(task_scheduler_guard);
            // send to background
            tokio::spawn(async move {
                let arc = arc1;
                let consumer = arc.try_read().unwrap();
                match consumer.run_task_core(arc2, task).await {
                    Ok(_) => {}
                    Err(e) => {
                        println!("running task failed, {:?}", &e);
                    }
                }
            });
        }
    }
}