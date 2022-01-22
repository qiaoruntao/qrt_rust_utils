use std::borrow::BorrowMut;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, trace};

use crate::db_task::task_consumer::{TaskConsumer, TaskConsumerResult};
use crate::db_task::task_scheduler::{TaskScheduler, TaskSchedulerError};
use crate::task::task::Task;

#[async_trait]
pub trait Runner<A, B> {
    async fn func(&self, task: Arc<RwLock<Task<A, B>>>) -> TaskConsumerResult;
    fn build_filter(&self) -> Option<Document> {
        None
    }
}

#[derive(Default)]
pub struct BasicConsumer<T, K, M: Runner<T, K>> {
    pub running_tasks: Mutex<Vec<Arc<RwLock<Task<T, K>>>>>,
    pub runner: M,
}

impl<
    ParamType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq,
    StateType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq,
    M: Runner<ParamType, StateType>,
> BasicConsumer<ParamType, StateType, M>
{
    pub fn new(runner: M) -> Self {
        BasicConsumer {
            running_tasks: Default::default(),
            runner,
        }
    }
}

#[async_trait]
impl<
    ParamType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq,
    StateType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq,
    RunnerType: Runner<ParamType, StateType> + Sync + Send,
> TaskConsumer<ParamType, StateType> for BasicConsumer<ParamType, StateType, RunnerType>
{
    fn build_filter(&self) -> Option<Document> {
        self.runner.build_filter()
    }

    async fn consume(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> TaskConsumerResult {
        info!("task {} start to run", &task.try_read().unwrap().key);
        let runner = &self.runner;
        runner.func(task).await
    }

    async fn add_running_task(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> bool {
        let mut guard = self.running_tasks.lock().await;
        guard.borrow_mut().push(task);
        true
    }

    async fn remove_running_task(
        &self,
        task2remove: Arc<RwLock<Task<ParamType, StateType>>>,
    ) -> bool {
        let mut guard = self.running_tasks.lock().await;
        let len = guard.len();
        let target_index = (0..len)
            .find(|&index| {
                let task = &guard[index];
                let current_task_guard = task.try_read().unwrap();
                let current_task = current_task_guard.deref();
                let target_task_guard = task2remove.try_read().unwrap();
                let target_task = target_task_guard.deref();
                current_task == target_task
            }).unwrap_or(len);
        if target_index == len {
            error!("failed to remove pending task");
            false
        } else {
            guard.swap_remove(target_index);
            true
        }
    }

    async fn run_maintainer(&self, task_scheduler: Arc<RwLock<TaskScheduler>>) {
        let guard = self.running_tasks.lock().await;
        let total_cnt = guard.len();
        if total_cnt == 0 {
            trace!("no task to maintain");
        } else {
            trace!("maintainer runs, task count= {}", guard.len());
        }

        let scheduler_guard = task_scheduler.try_read().unwrap();
        for running_task in guard.iter() {
            let update_result = scheduler_guard.update_task(running_task.clone()).await;
            match update_result {
                Ok(_) => {}
                Err(e) => {
                    let task_guard = running_task.read().await;
                    error!("run_maintainer {:?}, {:?}", &e, &task_guard.param);
                }
            }
        }
    }
}
