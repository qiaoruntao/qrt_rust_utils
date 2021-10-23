use std::collections::HashSet;
use std::time::Duration;

use mongodb::bson::Bson::ObjectId;
use mongodb::bson::Document;
use tracing::{error, info};

use crate::db_task::task_scheduler::{TaskScheduler, TaskSchedulerError};
use crate::task::task::Task;

pub enum TaskConsumerResult {
    Completed,
    Cancelled,
    Failed,
}

pub trait TaskConsumer<ParamType, StateType> {
    fn build_filter(&self) -> Option<Document>;
    // define how to consume a task
    fn consume(&self, task: &Task<ParamType, StateType>) -> TaskConsumerResult;
    // used to run background maintain
    fn add_running_task(&self, task: &Task<ParamType, StateType>) -> bool;
    fn remove_running_task(&self, task: &Task<ParamType, StateType>) -> bool;
    // continue to run a consumer
    fn run_consumer(&self, task_scheduler: &mut TaskScheduler) {
        loop {
            let task: Task<ParamType, StateType> = match task_scheduler.find_next_pending_task(self.build_filter()).await {
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
            // occupy the task
            match task_scheduler.occupy_pending_task(&task).await {
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
            let is_added = self.add_running_task(&task);
            if !is_added {
                error!("task {} already running", &task.key);
                continue;
            }
            // run the task
            let consumer_result = self.consume(&task);
            // remove maintainer
            let is_removed = self.remove_running_task(&task);
            if !is_removed {
                error!("task {} not exists", &task.key);
                continue;
            }
            // update result
            match consumer_result {
                TaskConsumerResult::Completed => {
                    task_scheduler.complete_task(&task).await;
                }
                TaskConsumerResult::Cancelled => {
                    task_scheduler.cancel_task(&task).await;
                }
                TaskConsumerResult::Failed => {
                    // TODO: do nothing, wait for timeout retry
                }
            }
        }
    }
}

pub struct WebcastConsumer<ParamType, StateType> {
    running_tasks: HashSet<String>,
}

impl<ParamType, StateType> TaskConsumer<ParamType, StateType> for WebcastConsumer<ParamType, StateType> {
    fn build_filter(&self) -> Option<Document> {
        None
    }

    fn consume(&self, task: &Task<ParamType, StateType>) -> TaskConsumerResult {
        info!("task {} consumed", task.key);
        TaskConsumerResult::Completed
    }

    fn add_running_task(&mut self, task: &Task<ParamType, StateType>) -> bool {
        self.running_tasks.insert(task.key.clone())
    }

    fn remove_running_task(&mut self, task: &Task<ParamType, StateType>) -> bool {
        self.running_tasks.remove(&task.key)
    }
}