use std::env;
use std::net::SocketAddr;
use std::time::Duration;

pub use console_subscriber;
// TODO
// #[cfg(feature = "tokio-debug")]
// use console_subscriber::spawn;
pub use tracing;
use tracing::{info, instrument, warn};
use tracing_subscriber::{EnvFilter, Layer, registry};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_logger(_application_name: &'static str, _rust_log_config: Option<&'static str>) {
    let filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::INFO.into());
    println!("filter={}", &filter);
    let registry = registry::Registry::default();
    if cfg!(feature="tokio-debug") {
        println!("enabling tokio-console");
        let listen_address = env::var("TokioConsoleAddr").unwrap_or("127.0.0.1:5555".into());
        let listen_address: SocketAddr = listen_address.parse().unwrap();
        let console_layer = console_subscriber::ConsoleLayer::builder()
            // set how long the console will retain data from completed tasks
            .retention(Duration::from_secs(60))
            // set the address the server is bound to
            .server_addr(listen_address)
            // ... other configurations ...
            .spawn();
        let filtered = fmt::layer()
            .with_timer(time::LocalTime::rfc_3339())
            .compact()
            .with_thread_names(true)
            .with_filter(filter);
        let subscriber = registry
            .with(console_layer)
            // .with(telemetry)
            .with(filtered); // log to stdout;
        tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
    } else {
        let filtered = fmt::layer()
            .with_timer(time::LocalTime::rfc_3339())
            .compact()
            .with_thread_names(true)
            .with_filter(filter);
        let subscriber = registry
            // .with(telemetry)
            .with(filtered); // log to stdout;
        tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
    };
}

#[instrument]
fn foo() {
    // let trace = register_dist_tracing_root(TraceId::default(), None);
    // println!("trace value: {:?}", trace);
    info!("test");
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::{foo, init_logger};

    #[tokio::test]
    async fn test_logger() {
        init_logger("test", None);
        foo();
        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}

