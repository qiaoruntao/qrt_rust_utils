use std::io;

use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::Registry;

use crate::config_manage::config_manager::ConfigManager;
use crate::logger::fluentd_layer::{FluentdLayer, FluentdLayerConfig};

pub struct Logger {}

impl Logger {
    pub fn generate_subscriber() -> Layered<BunyanFormattingLayer<MyMakeWriter>, Layered<JsonStorageLayer, Layered<FluentdLayer, Registry>>> {
        let formatting_layer = BunyanFormattingLayer::new(
            "tracing_demo".into(),
            MyMakeWriter {},
        );
        let config: FluentdLayerConfig = ConfigManager::read_config_with_directory("./config/logger").unwrap();
        let fluentd_layer = FluentdLayer::generate(&config);
        let subscriber = Registry::default()
            .with(fluentd_layer)
            .with(JsonStorageLayer)
            .with(formatting_layer);
        subscriber
    }

    pub fn init_logger() {
        let subscriber = Logger::generate_subscriber();
        tracing::subscriber::set_global_default(subscriber).expect("failed to set logger");
    }
}

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

#[cfg(test)]
mod test_logger {
    use tracing::instrument;

    use super::Logger;

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
        tracing::info!("Events have the full context of their parent span!");
    }

    #[test]
    fn span_test() {
        Logger::init_logger();
        tracing::info!("Orphan event without a parent span");
        a_unit_of_work(2);
    }
}