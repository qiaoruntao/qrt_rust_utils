use std::future::Future;
use std::time::Duration;

use async_recursion::async_recursion;

#[async_recursion]
async fn set_interval_inner<F, K>(func: F, duration: Duration) -> ()
where
    F: Fn() -> K + Send + 'static + Sync,
{
    // println!("set_interval_inner");
    tokio::time::sleep(duration).await;
    func();
    set_interval_inner(func, duration).await;
}

#[async_recursion]
async fn set_interval_async_inner<F, T>(func: F, duration: Duration) -> ()
where
    F: Fn() -> T + Send + 'static + Sync,
    T: Future + Send + 'static,
{
    // println!("set_interval_async_inner");
    tokio::time::sleep(duration).await;
    func().await;
    set_interval_async_inner(func, duration).await;
}

pub trait SetInterval {
    //noinspection RsSelfConvention
    fn set_interval<F, K>(&self, func: F, duration: Duration) -> ()
    where
        F: (Fn() -> K) + Send + 'static + Sync;
    //noinspection RsSelfConvention
    fn set_interval_async<F, T>(&self, func: F, duration: Duration) -> ()
    where
        F: (Fn() -> T) + Send + 'static + Sync,
        T: Future + Send + 'static;
}

impl SetInterval for tokio::runtime::Runtime {
    /// sync version of set_interval
    fn set_interval<F, K>(&self, func: F, duration: Duration) -> ()
    where
        F: (Fn() -> K) + Send + 'static + Sync,
    {
        self.spawn(async move {
            set_interval_inner(func, duration).await;
        });
    }

    /// async version of set_interval
    fn set_interval_async<F, T>(&self, func: F, duration: Duration) -> ()
    where
        F: (Fn() -> T) + Send + 'static + Sync,
        T: Future + Send + 'static,
    {
        self.spawn(async move {
            set_interval_async_inner(func, duration).await;
        });
    }
}

// TODO: find a way to deduplicate these parts
/// sync version of set_interval
pub fn set_interval<F, K>(func: F, duration: Duration) -> ()
where
    F: (Fn() -> K) + Send + 'static + Sync,
{
    tokio::spawn(async move {
        set_interval_inner(func, duration).await;
    });
}

/// async version of set_interval
pub fn set_interval_async<F, T>(func: F, duration: Duration) -> ()
where
    F: (Fn() -> T) + Send + 'static + Sync,
    T: Future + Send + 'static,
{
    tokio::spawn(async move {
        set_interval_async_inner(func, duration).await;
    });
}

#[cfg(test)]
mod test_set_interval {
    use std::time::Duration;

    use chrono::Local;

    use crate::set_interval::set_interval::{set_interval, set_interval_async, SetInterval};

    async fn fn_async() {
        println!("fn_async");
        let _result = tokio::spawn(async {
            println!("async {}", Local::now());
        })
        .await;
    }

    #[tokio::test]
    async fn test() {
        let duration = Duration::from_millis(100);
        set_interval(
            || {
                println!("sync {}", Local::now());
            },
            duration,
        );
        set_interval_async(fn_async, duration);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    #[test]
    fn test_runtime() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let duration = Duration::from_millis(100);
        runtime.set_interval(
            || {
                println!("sync {}", Local::now());
            },
            duration,
        );
        runtime.set_interval_async(fn_async, duration);

        // wait for 1 second
        runtime.block_on(async { tokio::time::sleep(Duration::from_secs(1)).await })
    }
}
