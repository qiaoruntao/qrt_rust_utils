use derive_builder::Builder;
use serde::Deserialize;
use serde::Serialize;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

pub struct Logger {}

#[derive(Debug, Builder, Serialize, Deserialize)]
pub struct LoggerConfig {
    #[serde(default = "default_level")]
    pub level: String,
}

fn default_level()->String{
    "info".to_string()
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig {
            level: default_level()
        }
    }
}

impl Logger {
    pub fn init_logger(logger_config: &LoggerConfig) {
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&logger_config.level))
            .unwrap();
        let fmt_layer = fmt::Layer::default()
            .with_timer(LocalTime::rfc_3339())
            .with_writer(std::io::stdout);
        let subscriber = Registry::default()
            // .with(JsonStorageLayer)
            .with(filter_layer)
            .with(fmt_layer);
        tracing::subscriber::set_global_default(subscriber).expect("failed to set logger");
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
