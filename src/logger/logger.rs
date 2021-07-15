use std::io;

use tracing_subscriber::fmt::MakeWriter;

pub struct Logger {

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
    use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
    use tracing_subscriber::{EnvFilter, Registry};
    use tracing_subscriber::layer::SubscriberExt;

    use crate::logger::logger::MyMakeWriter;

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

    #[test]
    fn basic_test() {
        let env_filter = EnvFilter::new("trace");
    }
}