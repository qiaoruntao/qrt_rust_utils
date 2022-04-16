use tracing::{info, instrument};
use tracing_honeycomb::{
    new_honeycomb_telemetry_layer, register_dist_tracing_root, SpanId,
    TraceId,
};
use tracing_subscriber::{EnvFilter, registry};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_logger(application_name: &'static str, rust_log_config: Option<&'static str>) {
    let telemetry_layer = {
        match option_env!("HoneycombKey") {
            None => {
                None
            }
            Some(honeycomb_key) => {
                let honeycomb_config = libhoney::Config {
                    options: libhoney::client::Options {
                        api_key: String::from(honeycomb_key),
                        dataset: "dag-cache".to_string(), // FIXME: rename if copying this example
                        ..libhoney::client::Options::default()
                    },
                    transmission_options: libhoney::transmission::Options::default(),
                };
                Some(new_honeycomb_telemetry_layer(application_name, honeycomb_config))
            }
        }
    };
    let mut filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::INFO.into());
    if let Some(str) = rust_log_config {
        for segment in str.split(',') {
            filter = filter.add_directive(segment.parse().expect("cannot parse rust log config"));
        }
    }
    let mut subscriber = registry::Registry::default() // provide underlying span data store
        .with(filter) // filter out low-level debug tracing (eg tokio executor)
        .with(tracing_subscriber::fmt::Layer::default()); // log to stdout;
    if let Some(telemetry_layer) = telemetry_layer {
        let subscriber = subscriber.with(telemetry_layer); // publish to honeycomb backend
        tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
    } else {
        tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
    }
}

#[instrument]
fn foo() {
    let trace = register_dist_tracing_root(TraceId::default(), None);
    println!("trace value: {:?}", trace);
    info!("test");
}

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
