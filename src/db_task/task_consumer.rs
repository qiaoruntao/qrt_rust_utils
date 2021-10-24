use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use mongodb::bson::Bson::ObjectId;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;
use serde::Serialize;
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
pub trait TaskConsumer<ParamType: Debug + Serialize + DeserializeOwned + Unpin + Sync + Send, StateType: Debug + Serialize + DeserializeOwned + Unpin + Sync + Send> {
    fn build_filter(&self) -> Option<Document>;
    // define how to consume a task
    fn consume(&self, task: Arc<RefCell<Task<ParamType, StateType>>>) -> TaskConsumerResult;
    // used to run background maintain
    fn add_running_task(&self, task: Arc<RefCell<Task<ParamType, StateType>>>) -> bool;
    fn remove_running_task(&self, task: Arc<RefCell<Task<ParamType, StateType>>>) -> bool;
    // maintain task
    fn run_maintainer(&self, task_scheduler: &TaskScheduler);
    // continue to run a consumer
    async fn run_consumer(&'static self, task_scheduler: &'static TaskScheduler) {
        self.run_maintainer(task_scheduler);
        let filter = self.build_filter();
        loop {
            // find a task
            let result = task_scheduler.find_next_pending_task(filter.clone()).await;
            let task = match result {
                Ok(task) => task,
                Err(e) => {
                    match e {
                        TaskSchedulerError::NoPendingTask => {
                            // wait a while
                            // tokio::time::sleep(Duration::from_secs(5)).await;
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
            tokio::spawn(self.run_task_core(task_scheduler, task));
        };
    }

    async fn run_task_core(&self, task_scheduler: &TaskScheduler, task: Arc<RefCell<Task<ParamType, StateType>>>) -> Result<(), TaskSchedulerError> {
        // occupy the task
        match task_scheduler.occupy_pending_task::<ParamType, StateType>(task.clone()).await {
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
        let x: &RefCell<Task<ParamType, StateType>> = task.borrow();
        let key = x.borrow().key.clone();
        let is_added = self.add_running_task(task);
        if !is_added {
            error!("task {} already running", &key);
            return Err(TaskSchedulerError::MaintainerError);
        }
        // check if we are running maintainer

        // run the task
        let consumer_result = self.consume(task.clone());
        // remove maintainer
        let is_removed = self.remove_running_task(task.clone());
        if !is_removed {
            error!("task {} not exists", &key);
            return Err(TaskSchedulerError::MaintainerError);
        }
        // update result
        match consumer_result {
            TaskConsumerResult::Completed => {
                task_scheduler.complete_task(&x.deref()).await
            }
            TaskConsumerResult::Cancelled => {
                task_scheduler.cancel_task(&x.deref()).await
            }
            TaskConsumerResult::Failed => {
                // TODO: do nothing, wait for timeout retry
                Err(TaskSchedulerError::TaskFailedError)
            }
        }
    }
}

pub struct WebcastConsumer<ParamType, StateType> {
    running_tasks: Mutex<Vec<Arc<RefCell<Task<ParamType, StateType>>>>>,
}

impl<ParamType: Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq, StateType: Debug + Send + Serialize + DeserializeOwned + Unpin + Sync + PartialEq> TaskConsumer<ParamType, StateType> for WebcastConsumer<ParamType, StateType> {
    fn build_filter(&self) -> Option<Document> {
        None
    }

    fn consume(&self, task: Arc<RefCell<Task<ParamType, StateType>>>) -> TaskConsumerResult {
        // info!("task {} consumed", &task.borrow().borrow().key);
        TaskConsumerResult::Completed
    }

    fn add_running_task(&self, task: Arc<RefCell<Task<ParamType, StateType>>>) -> bool {
        let guard = self.running_tasks.lock().unwrap();
        guard.borrow().push(task);
        true
    }

    fn remove_running_task(&self, task2remove: Arc<RefCell<Task<ParamType, StateType>>>) -> bool {
        let mut guard = self.running_tasks.lock().unwrap().borrow();
        for (index, task) in guard.iter().enumerate() {
            let x: &RefCell<Task<ParamType, StateType>> = task.borrow().deref();
            let x1: &RefCell<Task<ParamType, StateType>> = task2remove.borrow().deref();
            if *x == *x1 {
                guard.swap_remove(index);
                return true;
            }
        }
        false
    }

    fn run_maintainer(&self, task_scheduler: &TaskScheduler) {
        tokio::spawn(async {
            let start = Instant::now() + Duration::from_millis(50);
            let mut interval = interval_at(start, Duration::from_secs(10));
            loop {
                interval.tick().await;
                for running_task in self.running_tasks.lock().unwrap().borrow().iter() {
                    task_scheduler.occupy_pending_task(running_task.clone()).await;
                }
            };
        });
    }
}