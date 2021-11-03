use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::RwLock;
use tracing::error;

use crate::db_task::task_scheduler::{TaskScheduler, TaskSchedulerError};
use crate::task::task::Task;

pub enum TaskConsumerResult {
    Completed,
    Cancelled,
    Failed,
}

#[async_trait]
pub trait TaskConsumer<
    ParamType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send,
    StateType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send,
>
{
    fn build_filter(&self) -> Option<Document>;
    // define how to consume a task
    async fn consume(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> TaskConsumerResult;
    // used to run background maintain
    async fn add_running_task(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> bool;
    async fn remove_running_task(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> bool;
    // maintain task
    async fn run_maintainer(&self, task_scheduler: Arc<RwLock<TaskScheduler>>);

    async fn run_task_core(
        &self,
        task_scheduler: Arc<RwLock<TaskScheduler>>,
        task: Arc<RwLock<Task<ParamType, StateType>>>,
    ) -> Result<(), TaskSchedulerError> {
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
            TaskConsumerResult::Completed => task_scheduler_guard.complete_task(task.clone()).await,
            TaskConsumerResult::Cancelled => task_scheduler_guard.cancel_task(task.clone()).await,
            TaskConsumerResult::Failed => {
                // TODO: do nothing, wait for timeout retry
                Err(TaskSchedulerError::TaskFailedError)
            }
        }
    }
}
