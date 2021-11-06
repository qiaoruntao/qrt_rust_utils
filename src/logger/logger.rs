use derive_builder::Builder;
use serde::Deserialize;
use serde::Serialize;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::config_manage::config_manager::ConfigManager;
use crate::logger::fluentd_layer::{FluentdLayer, FluentdLayerConfig};

pub struct Logger {}

#[derive(Debug, Builder, Serialize, Deserialize)]
pub struct LoggerConfig {
    pub with_fluentd: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig {
            // disable fluentd because it has severe problem
            with_fluentd: false,
        }
    }
}

impl Logger {
    pub fn init_logger(logger_config: &LoggerConfig) {
        let fmt_layer = fmt::Layer::default().with_timer(LocalTime::rfc_3339());
        let subscriber = Registry::default()
            // .with(JsonStorageLayer)
            .with(fmt_layer);
        if logger_config.with_fluentd {
            let config: FluentdLayerConfig =
                ConfigManager::read_config_with_directory("./config/logger").unwrap();
            let fluentd_layer = FluentdLayer::generate(&config);
            tracing::subscriber::set_global_default(subscriber.with(fluentd_layer))
                .expect("failed to set logger");
        } else {
            tracing::subscriber::set_global_default(subscriber).expect("failed to set logger");
        };
    }
}

#[cfg(test)]
mod test_logger {
    use tracing::instrument;

    use crate::logger::logger::LoggerConfig;

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
        Logger::init_logger(&LoggerConfig::default());
        tracing::info!("Orphan event without a parent span");
        a_unit_of_work(2);
    }
}
