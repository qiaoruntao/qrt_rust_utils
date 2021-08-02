use std::future::Future;
use std::time::Duration;

use async_recursion::async_recursion;

#[async_recursion]
async fn set_interval_inner<F, K>(func: F) -> ()
    where F: Fn() -> K + Send + 'static + Sync {
    println!("set_interval_inner");
    tokio::time::sleep(Duration::from_millis(100)).await;
    func();
    set_interval_inner(func).await;
}

#[async_recursion]
async fn set_interval_async_inner<F, T>(func: F) -> ()
    where F: Fn() -> T + Send + 'static + Sync, T: Future + Send + 'static {
    println!("set_interval_async_inner");
    tokio::time::sleep(Duration::from_millis(100)).await;
    func().await;
    set_interval_async_inner(func).await;
}

pub fn set_interval<F, K>(func: F) -> ()
    where F: (Fn() -> K) + Send + 'static + Sync {
    tokio::spawn(async move {
        set_interval_inner(func).await;
    });
}

pub fn set_interval_async<F, T>(func: F) -> ()
    where F: (Fn() -> T) + Send + 'static + Sync, T: Future + Send + 'static {
    tokio::spawn(async move {
        set_interval_async_inner(func).await;
    });
}

#[cfg(test)]
mod test_set_interval {
    use std::time::Duration;

    use chrono::Local;

    use crate::set_interval::set_interval::{set_interval, set_interval_async};

    async fn fn_async() {
        println!("fn_async");
        let _result = tokio::spawn(async {
            println!("async {}", Local::now());
        }).await;
    }

    #[tokio::test]
    async fn test() {
        set_interval(|| {
            println!("sync {}", Local::now());
        });
        set_interval_async(fn_async);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
