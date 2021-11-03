use std::borrow::BorrowMut;
use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;
use mongodb::bson::Document;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use tracing::info;

use crate::db_task::task_consumer::{TaskConsumer, TaskConsumerResult};
use crate::db_task::task_scheduler::TaskScheduler;
use crate::task::task::Task;

pub struct BasicConsumer<T, K, F> {
    running_tasks: Mutex<Vec<Arc<RwLock<Task<T, K>>>>>,
    f: F,
}

impl<T, K, F> BasicConsumer<T, K, F> {
    pub fn new<M>(func: F) -> Self
        where
            F: Fn(Arc<RwLock<Task<T, K>>>) -> M,
            M: Future<Output=TaskConsumerResult> + Sync + Send + 'static, {
        BasicConsumer {
            running_tasks: Default::default(),
            f: func,
        }
    }
}

#[async_trait]
impl<
    ParamType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq,
    StateType: 'static + Debug + Serialize + DeserializeOwned + Unpin + Sync + Send + PartialEq,
    F: Fn(Arc<RwLock<Task<ParamType, StateType>>>) -> M + Sync + Send,
    M: Future<Output=TaskConsumerResult> + Sync + Send + 'static
> TaskConsumer<ParamType, StateType> for BasicConsumer<ParamType, StateType, F> {
    fn build_filter(&self) -> Option<Document> {
        None
    }

    async fn consume(&self, task: Arc<RwLock<Task<ParamType, StateType>>>) -> TaskConsumerResult {
        info!("task {} consumed", &task.try_read().unwrap().key);
        let f = &self.f;
        f(task).await
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
        let guard = self.running_tasks.lock().await;
        info!("maintainer runs, task count= {}", guard.len());
        let scheduler_guard = task_scheduler.try_read().unwrap();
        for running_task in guard.iter() {
            let update_result = scheduler_guard.update_task(running_task.clone()).await;
            match update_result {
                Ok(_) => {}
                Err(e) => {
                    dbg!("{:?}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod test_task_scheduler {
    use std::sync::Arc;

    use tokio::sync::RwLock;
    use tokio::time::Duration;
    use tracing::error;
    use tracing::info;

    use crate::config_manage::config_manager::ConfigManager;
    use crate::db_task::basic_consumer::BasicConsumer;
    use crate::db_task::task_consumer::{TaskConsumer, TaskConsumerResult};
    use crate::db_task::task_scheduler::{TaskScheduler, TaskSchedulerError};
    use crate::logger::logger::Logger;
    use crate::mongodb_manager::mongodb_manager::MongoDbManager;
    use crate::task::task::Task;

    async fn func(_task: Arc<RwLock<Task<i32, i32>>>) -> TaskConsumerResult {
        TaskConsumerResult::Completed
    }

    #[tokio::test]
    async fn test() {
        let logger_config = ConfigManager::read_config_with_directory("./config/logger").unwrap();
        Logger::init_logger(&logger_config);
        let mongodb_config = ConfigManager::read_config_with_directory("./config/mongo").unwrap();
        let db_manager = MongoDbManager::new(mongodb_config, "Logger").unwrap();
        let scheduler = TaskScheduler::new(db_manager, "live_record".into());
        let webcast_consumer = BasicConsumer::new(func);
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
            let result = task_scheduler_guard
                .find_next_pending_task(filter.clone())
                .await;
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
                        error!("unknown error while occupying task {:?}", e);
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
