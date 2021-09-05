use qrt_rust_utils::logger::logger::{Logger, LoggerConfig};

fn main() {
    Logger::init_logger(&LoggerConfig::default());
    tracing::info!("test info");
}

#[cfg(test)]
mod test_main {
    use qrt_rust_utils::logger::logger::{Logger, LoggerConfig};

    #[test]
    fn test_logger() {
        Logger::init_logger(&LoggerConfig::default());
        tracing::info!("test info");
    }
}
