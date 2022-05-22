use std::env;
use std::net::SocketAddr;
use std::time::Duration;

pub use console_subscriber;
// TODO
// #[cfg(feature = "tokio-debug")]
// use console_subscriber::spawn;
pub use tracing;
use tracing::{info, instrument, warn};
use tracing_honeycomb::{
    new_honeycomb_telemetry_layer, register_dist_tracing_root, TraceId,
};
pub use tracing_honeycomb;
use tracing_subscriber::{EnvFilter, Layer, registry};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_logger(application_name: &'static str, _rust_log_config: Option<&'static str>) {
    let telemetry_layer = if cfg!(feature="honeycomb-logging") {
        match env::var("HoneycombKey") {
            Err(e) => {
                // no logger now
                eprintln!("cannot get HoneycombKey {}", &e);
                None
            }
            Ok(honeycomb_key) => {
                let honeycomb_config = libhoney::Config {
                    options: libhoney::client::Options {
                        api_key: honeycomb_key,
                        dataset: "dag-cache".to_string(), // FIXME: rename if copying this example
                        ..libhoney::client::Options::default()
                    },
                    transmission_options: libhoney::transmission::Options::default(),
                };
                Some(new_honeycomb_telemetry_layer(application_name, honeycomb_config))
            }
        }
    } else {
        None
    };
    let filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::INFO.into());
    let registry = registry::Registry::default();
    // TODO; need better solution
    if let Some(telemetry_layer) = telemetry_layer {
        println!("init logger with telemetry now");
        if cfg!(feature="tokio-debug") {
            println!("enabling tokio-console");
            let listen_address = env::var("TokioConsoleAddr").unwrap_or("127.0.0.1:5555".into());
            let listen_address: SocketAddr = listen_address.parse().unwrap();
            let console_layer = console_subscriber::ConsoleLayer::builder()
                // set how long the console will retain data from completed tasks
                .retention(Duration::from_secs(10))
                // set the address the server is bound to
                .server_addr(listen_address)
                // ... other configurations ...
                .spawn();
            let subscriber = registry
                .with(console_layer)
                .with(fmt::layer().with_filter(filter))
                .with(telemetry_layer);
            tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
        } else {
            let subscriber = registry
                .with(fmt::layer().with_filter(filter))
                .with(telemetry_layer); // log to stdout;
            tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
        };
    } else {
        println!("init logger without telemetry now");
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
            let subscriber = registry
                .with(console_layer)
                .with(fmt::layer().with_filter(filter)); // log to stdout;
            tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
        } else {
            let subscriber = registry
                .with(fmt::layer().with_filter(filter)); // log to stdout;
            tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
        };
    };
}

#[instrument]
fn foo() {
    let trace = register_dist_tracing_root(TraceId::default(), None);
    println!("trace value: {:?}", trace);
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

