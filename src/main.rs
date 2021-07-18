use qrt_rust_utils::logger::logger::Logger;

fn main() {
    Logger::init_logger();
    tracing::info!("test info");
}

#[cfg(test)]
mod test_main {
    use qrt_rust_utils::logger::logger::Logger;

    #[test]
    fn test_logger() {
        Logger::init_logger();
        tracing::info!("test info");
    }
}