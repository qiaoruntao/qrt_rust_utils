#[allow(ambiguous_associated_items)]
use std::fmt::Debug;

use chrono::{DateTime, Local};
use fruently::fluent::Fluent;
use fruently::forwardable::JsonForwardable;
use fruently::retry_conf::RetryConf;
use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use tracing::{Event, Level, Subscriber};
use tracing::field::{Field, Visit};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FluentdLogLocation {
    #[serde(rename = "lineNumber")]
    line_number: Option<u32>,
    #[serde(rename = "filename")]
    filename: Option<String>,
}

#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug)]
pub enum FluentdLogLevel {
    #[serde(rename = "silly")]
    Silly,
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    // TODO: #[allow(ambiguous_associated_items)]
    ERROR,
    #[serde(rename = "fatal")]
    Fatal,
}

impl From<&Level> for FluentdLogLevel {
    fn from(value: &Level) -> Self {
        match *value {
            Level::TRACE => {
                FluentdLogLevel::Trace
            }
            Level::DEBUG => {
                FluentdLogLevel::Debug
            }
            Level::ERROR => {
                FluentdLogLevel::ERROR
            }
            Level::INFO => {
                FluentdLogLevel::Info
            }
            Level::WARN => {
                FluentdLogLevel::Warn
            }
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FluentdLogContent {
    #[serde(rename = "rawValue")]
    raw_value: String,
}

// the object we write to fluentd
// #[serde_with::serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FluentdLog {
    #[serde(rename = "createDate")]
    pub create_date: DateTime<Local>,
    #[serde(rename = "instanceName")]
    pub instance_name: String,
    #[serde(rename = "loggerName")]
    pub logger_name: String,
    #[serde(rename = "logLevel")]
    pub log_level: FluentdLogLevel,
    pub content: FluentdLogContent,
    pub tags: Vec<String>,
    pub location: FluentdLogLocation,
}

pub struct FluentdLayer {
    config: FluentdLayerConfig,
}

#[derive(Debug)]
pub struct TestVisitor {
    message: Option<String>,
}

impl Visit for TestVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.into());
        }
        // dbg!(field, value);
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
        // dbg!(field, value);
    }
}

impl<S: Subscriber> Layer<S> for FluentdLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        // dbg!(event);
        // try to get the message
        let mut visitor = TestVisitor { message: None };
        event.record(&mut visitor);
        let message = visitor.message.unwrap_or_default();

        // dbg!(field);
        // generate the log object
        let fluentd_log = FluentdLog {
            create_date: Local::now(),
            instance_name: "test".to_string(),
            logger_name: "default".to_string(),
            log_level: metadata.level().into(),
            content: FluentdLogContent { raw_value: message },
            tags: vec![],
            location: FluentdLogLocation { line_number: metadata.line(), filename: metadata.file().map(|name| name.to_string()) },
        };
        let fluentd_layer = FluentdLayer::generate(&self.config);
        fluentd_layer.send_log(&fluentd_log);
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct FluentdLayerConfig {
    pub server_url: String,
    pub tag: String,
    pub instance_name: String,
}

impl FluentdLayer {
    pub fn send_log(&self, fluentd_log: &FluentdLog) {
        // post method consumes instance
        println!("start send_log");
        let config = &self.config;
        let retry_conf = RetryConf::new().max(1).multiplier(0.0);
        let fluentd_instance = Fluent::new_with_conf(
            config.server_url.clone(),
            config.tag.clone(),
            retry_conf,
        );
        let _result = fluentd_instance.post(fluentd_log);
        dbg!(&_result);
        // assert!(result.is_ok())
    }
    pub fn generate(config: &FluentdLayerConfig) -> FluentdLayer {
        FluentdLayer {
            config: config.clone()
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::Local;

    use crate::config_manage::config_manager::ConfigManager;
    use crate::logger::fluentd_layer::{FluentdLayer, FluentdLayerConfig, FluentdLog, FluentdLogContent, FluentdLogLevel, FluentdLogLocation};

    #[test]
    fn send_test() {
        let fluentd_log = FluentdLog {
            create_date: Local::now(),
            instance_name: "test".to_string(),
            logger_name: "test_logger".to_string(),
            log_level: FluentdLogLevel::Silly,
            content: FluentdLogContent { raw_value: "test content".to_string() },
            tags: vec!["aaa".to_string()],
            location: FluentdLogLocation { line_number: None, filename: None },
        };
        let fluentd_layer = generate_layer();
        fluentd_layer.send_log(&fluentd_log);
    }

    fn generate_layer() -> FluentdLayer {
        let config: FluentdLayerConfig = ConfigManager::read_config_with_directory("./config/logger").unwrap();
        let fluentd_layer = FluentdLayer::generate(&config);
        fluentd_layer
    }
}
