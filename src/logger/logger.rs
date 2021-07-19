use std::io::Stdout;

use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::{DefaultFields, Format, Full};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::fmt::time::{ChronoLocal, ChronoUtc, SystemTime};
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::Registry;

use crate::config_manage::config_manager::ConfigManager;
use crate::logger::fluentd_layer::{FluentdLayer, FluentdLayerConfig};

pub struct Logger {}

impl Logger {
    pub fn generate_subscriber() -> Layered<Layer<Layered<FluentdLayer, Registry>, DefaultFields, Format<Full, ChronoLocal>, fn() -> Stdout>, Layered<FluentdLayer, Registry>> {
        // let formatting_layer = BunyanFormattingLayer::new(
        //     "tracing_demo".into(),
        //     MyMakeWriter {},
        // );
        let config: FluentdLayerConfig = ConfigManager::read_config_with_directory("./config/logger").unwrap();
        let fluentd_layer = FluentdLayer::generate(&config);
        let fmt_layer = fmt::Layer::default()
            .with_timer(ChronoLocal::rfc3339());
        let subscriber = Registry::default()
            // .with(JsonStorageLayer)
            .with(fluentd_layer)
            .with(fmt_layer);
        subscriber
    }

    pub fn init_logger() {
        let subscriber = Logger::generate_subscriber();
        tracing::subscriber::set_global_default(subscriber).expect("failed to set logger");
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