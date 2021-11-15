use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::future::err;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{RwLock, Semaphore, SemaphorePermit, TryAcquireError};
use tracing::{error, info, trace};

use crate::db_task::consumer_config::ConsumerConfig;
use crate::db_task::task_scheduler::{TaskScheduler, TaskSchedulerError};
use crate::task::task::Task;

pub enum TaskConsumerResult {
    Completed,
    Cancelled,
    Failed,
}

pub trait TaskParamType:
Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq
{}

pub trait TaskStateType: Debug + Serialize + DeserializeOwned + Unpin + Sync + Send {}

#[async_trait]
pub trait TaskConsumer<
    ParamType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq,
    StateType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq,
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
    async fn main_loop<T: 'static + TaskConsumer<ParamType, StateType> + Sync + Send>(
        arc_scheduler: Arc<RwLock<TaskScheduler>>,
        arc_consumer: Arc<RwLock<T>>,
        consumer_config: ConsumerConfig,
    ) {
        info!("start to run main loop");
        let semaphore = Arc::new(Semaphore::new(consumer_config.max_concurrency as usize));
        loop {
            // try get token now
            let arc1 = arc_consumer.clone();
            let consumer = arc1.try_read().unwrap();
            let filter = consumer.build_filter();
            // maintain
            let arc2 = arc_scheduler.clone();
            consumer.run_maintainer(arc2.clone()).await;
            drop(consumer);
            let token = match semaphore.clone().try_acquire_owned() {
                Ok(token) => { token }
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };
            // find a task
            let task_scheduler_guard = arc2.try_read().unwrap();
            // TODO: merge find and occupy for relentless check
            let result = task_scheduler_guard
                .find_next_pending_task(filter.clone())
                .await;
            let task = match result {
                Ok(task) => task,
                Err(e) => {
                    match e {
                        TaskSchedulerError::NoPendingTask => {
                            trace!("no pending task found");
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
                        error!("unknown error while occupying task {:?}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
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
                drop(token);
            });
            // TODO: test
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
