use std::io;

use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::Layer;

pub struct Logger {}

pub struct MyWriter {}

impl io::Write for MyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        print!("{}", String::from_utf8(Vec::from(buf)).unwrap());
        std::io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        std::io::stdout().flush()
    }
}

pub struct MyMakeWriter {}

impl MakeWriter for MyMakeWriter {
    type Writer = MyWriter;

    fn make_writer(&self) -> Self::Writer {
        MyWriter {}
    }
}

pub struct MyLayer {}

#[cfg(test)]
mod test_logger {
    use std::time::Duration;

    use tracing::instrument;
    use tracing::level_filters::LevelFilter;
    use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
    use tracing_honeycomb::{current_dist_trace_ctx, new_honeycomb_telemetry_layer, register_dist_tracing_root, SpanId, TraceId};
    use tracing_subscriber::{EnvFilter, Registry};
    use tracing_subscriber::layer::SubscriberExt;

    use crate::logger::logger::MyMakeWriter;
    use libhoney::{init, Value};
    use std::collections::HashMap;

    #[instrument]
    pub fn a_unit_of_work(first_parameter: u64) {
        for i in 0..2 {
            a_sub_unit_of_work(i);
        }
        tracing::info!(excited = "true", "Tracing is quite cool!");
        tracing::info!("Tracing is quite bad!");
    }

    #[instrument]
    pub fn a_sub_unit_of_work(sub_parameter: u64) {
        // let (trace_id, span_id) = current_dist_trace_ctx().unwrap();
        // println!("{}, {}", trace_id, span_id);
        tracing::info!("Events have the full context of their parent span!");
    }

    #[test]
    fn span_test() {
        let formatting_layer = BunyanFormattingLayer::new(
            "tracing_demo".into(),
            MyMakeWriter {},
        );
        let subscriber = Registry::default()
            .with(JsonStorageLayer)
            .with(formatting_layer);
        tracing::subscriber::set_global_default(subscriber).unwrap();

        tracing::info!("Orphan event without a parent span");
        a_unit_of_work(2);
    }

    #[tokio::test]
    #[instrument]
    async fn honeycomb_test() {
        let mut options = libhoney::transmission::Options::default();
        // options.batch_timeout = Duration::from_secs(1);
        // options.max_batch_size = 1;
        let honeycomb_config = libhoney::Config {
            options: generate_config(),
            transmission_options: options,
        };

        let telemetry_layer = new_honeycomb_telemetry_layer("my-service-name", honeycomb_config);
// NOTE: the underlying subscriber MUST be the Registry subscriber
        let subscriber = Registry::default() // provide underlying span data store
            // .with(LevelFilter::TRACE) // filter out low-level debug tracing (eg tokio executor)
            .with(tracing_subscriber::fmt::Layer::default()) // log to stdout
            .with(telemetry_layer); // publish to honeycomb backend
        tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");
        register_dist_tracing_root(TraceId::new(), None).unwrap_err();
        tokio::time::sleep(Duration::from_secs(1)).await;

        tracing::info!("first log");
        a_sub_unit_of_work(1);
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    fn generate_config() -> Options {
        libhoney::client::Options {
            api_key: String::from("sample_key"),
            dataset: "my-dataset-name".to_string(),
            ..libhoney::client::Options::default()
        }
    }

    use libhoney::json;
    use libhoney::client::Options;

    #[test]
    fn libhoney_test() {
        use libhoney::FieldHolder; // Add trait to allow for adding fields
// Call init to get a client
        let mut client = init(libhoney::Config {
            options: generate_config(),
            transmission_options: libhoney::transmission::Options::default(),
        });

        let mut data: HashMap<String, Value> = HashMap::new();
        data.insert("duration_ms".to_string(), json!(153.12));
        data.insert("method".to_string(), Value::String("get".to_string()));
        data.insert("hostname".to_string(), Value::String("appserver15".to_string()));
        data.insert("payload_length".to_string(), json!(27));

        let mut ev = client.new_event();
        ev.add(data);
        // In production code, please check return of `.send()`
        ev.send(&mut client).err();
        ev.send(&mut client).err();
        ev.send(&mut client).err();
        client.close().unwrap();
        // tokio::time::sleep(Duration::from_secs(10)).await;
    }
}