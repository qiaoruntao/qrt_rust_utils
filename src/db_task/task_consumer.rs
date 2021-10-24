use std::borrow::{Borrow, BorrowMut};
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
    fn consume(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> TaskConsumerResult;
    // used to run background maintain
    async fn add_running_task(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> bool;
    async fn remove_running_task(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> bool;
    // maintain task
    async fn run_maintainer(&self, task_scheduler: Arc<RwLock<TaskScheduler>>);
    // continue to run a consumer
    async fn run_consumer(&'static self, task_scheduler: Arc<RwLock<TaskScheduler>>) {
        let filter = self.build_filter();
        loop {
            // maintain
            self.run_maintainer(task_scheduler.clone());
            // find a task
            let guard = task_scheduler.try_read().unwrap();
            let result = guard.find_next_pending_task(filter.clone()).await;
            let task = match result {
                Ok(task) => task,
                Err(e) => {
                    match e {
                        TaskSchedulerError::NoPendingTask => {
                            // wait a while
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                        _ => {
                            // unexpected error
                            error!("{:?}", e);
                            break;
                        }
                    }
                }
            };
            // send to background
            tokio::spawn(self.run_task_core(task_scheduler.clone(), task));
        };
    }

    async fn run_task_core(&self, task_scheduler: Arc<RwLock<TaskScheduler>>, task: Arc<RwLock<Task<ParamType, StateType>>>) -> Result<(), TaskSchedulerError> {
        // occupy the task
        let guard = task_scheduler.try_read().unwrap();
        match guard.occupy_pending_task::<ParamType, StateType>(task.clone()).await {
            Err(e) => {
                match e {
                    TaskSchedulerError::OccupyTaskFailed | TaskSchedulerError::NoMatchedTask => {
                        // this is ok
                    }
                    _ => {}
                }
            }
            Ok(_) => {
                // occupied
            }
        }
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
        match consumer_result {
            TaskConsumerResult::Completed => {
                guard.complete_task(task.clone()).await
            }
            TaskConsumerResult::Cancelled => {
                guard.cancel_task(task.clone()).await
            }
            TaskConsumerResult::Failed => {
                // TODO: do nothing, wait for timeout retry
                Err(TaskSchedulerError::TaskFailedError)
            }
        }
    }
}

pub struct WebcastConsumer {
    running_tasks: Mutex<Vec<Arc<RwLock<Task<(), ()>>>>>,
}

#[async_trait]
impl<ParamType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq, StateType: 'static + Debug + Send + Serialize + DeserializeOwned + Unpin + Sync + PartialEq> TaskConsumer<ParamType, StateType> for WebcastConsumer {
    fn build_filter(&self) -> Option<Document> {
        None
    }

    fn consume(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> TaskConsumerResult {
        info!("task {} consumed", &task.try_read().unwrap().key);
        TaskConsumerResult::Completed
    }

    async fn add_running_task(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> bool {
        let mut guard = self.running_tasks.lock().await;
        guard.borrow_mut().push(task);
        true
    }

    async fn remove_running_task(&self, task2remove: Arc<RwLock<Task<ParamType, StateType>>>) -> bool {
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
        info!("maintainer runs");
        tokio::time::sleep(Duration::from_secs(10));
        let guard = self.running_tasks.lock().await;
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

    use tokio::sync::RwLock;

    use crate::config_manage::config_manager::ConfigManager;
    use crate::db_task::task_consumer::{TaskConsumer, WebcastConsumer};
    use crate::db_task::task_scheduler::TaskScheduler;
    use crate::mongodb_manager::mongodb_manager::MongoDbManager;

    #[test]
    fn test() {
        let result = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(result, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager);
        let webcast_consumer = WebcastConsumer {
            running_tasks: Default::default()
        };
        webcast_consumer.run_consumer(Arc::new(RwLock::new(scheduler)));
    }
}