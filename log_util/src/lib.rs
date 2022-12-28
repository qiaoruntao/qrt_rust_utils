use std::env;
use std::net::SocketAddr;
use std::time::Duration;
use time::UtcTime;
pub use console_subscriber;
use opentelemetry::KeyValue;
use opentelemetry::sdk::{Resource, trace};
use opentelemetry::sdk::trace::Sampler;
use opentelemetry_otlp::WithExportConfig;
use tonic::metadata::*;
// TODO
// #[cfg(feature = "tokio-debug")]
// use console_subscriber::spawn;
pub use tracing;
use tracing::{info, instrument, Level, warn};
use tracing_subscriber::{EnvFilter, filter, Layer, registry};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_logger(application_name: &'static str, _rust_log_config: Option<&'static str>) {
    let filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::INFO.into());
    println!("filter={}", &filter);
    let registry = registry::Registry::default();
    let mut map = MetadataMap::with_capacity(3);
    let api_key = match env::var("OTLP_KEY") {
        Ok(val) => val,
        Err(_) => panic!("api key not found"),
    };
    // map.insert("api-key", api_key.parse().unwrap());
    map.insert("x-honeycomb-team", api_key.parse().unwrap());
    map.insert("x-honeycomb-dataset", "rust".parse().unwrap());
    let application_name = application_name;
    println!("application name={}", application_name);
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                // .with_endpoint("https://otlp.nr-data.net")
                // .with_endpoint("http://localhost:4317")
                .with_endpoint("https://api.honeycomb.io:443")
                .with_metadata(map)
                .with_timeout(Duration::from_secs(3))
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                // .with_id_generator(RandomIdGenerator::default())
                // .with_max_events_per_span(64)
                // .with_max_attributes_per_span(16)
                // .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new("service.name", application_name)])),
        )
        .install_batch(opentelemetry::runtime::Tokio).unwrap();
    let telemetry_filter = filter::Targets::new()
        .with_target("h2", Level::WARN)
        .with_default(Level::INFO);
// Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(telemetry_filter);

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
        // let offset = time::UtcOffset::current_local_offset().expect("should get local offset!");
        // let time_format = time::format_description::parse("[hour]:[minute]:[second]")
        //     .expect("format string should be valid!");
        // let timer = OffsetTime::new(offset, "ddddd");
        let filtered = fmt::layer()
            // .with_timer(timer)
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
            // .with_timer(time::LocalTime::rfc_3339())
            .compact()
            .with_thread_names(true)
            .with_filter(filter);
        let subscriber = registry
            .with(telemetry)
            .with(filtered); // log to stdout;
        tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
    };
}

#[instrument]
fn foo() {
    // let trace = register_dist_tracing_root(TraceId::default(), None);
    // println!("trace value: {:?}", trace);
    info!("test");
    bar();
}

#[instrument]
fn bar() {
    // let trace = register_dist_tracing_root(TraceId::default(), None);
    // println!("trace value: {:?}", trace);
    info!("test2");
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tracing::Level;
    use tracing::span;

    use crate::{foo, init_logger};

    #[tokio::test]
    async fn test_logger() {
        init_logger("test_logger", None);
        for _ in 0..100 {
            let span = span!(Level::INFO, "my_span");
            let _guard = span.enter();
            foo();
        }
        tokio::time::sleep(Duration::from_secs(4)).await;
    }
}

