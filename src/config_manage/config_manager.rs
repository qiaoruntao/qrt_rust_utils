use std::path::Path;

use config::{Config, ConfigError, File};
use serde::de::DeserializeOwned;

pub struct ConfigManager {}

impl ConfigManager {
    pub fn read_config_with_directory<T: DeserializeOwned>(
        config_directory: &str,
    ) -> Result<T, ConfigError> {
        let default_file_path = Path::new(config_directory).join("default-example.toml");
        let custom_file_path = Path::new(config_directory).join("custom.toml");
        let mut s = Config::new();
        // load default
        s.merge(File::from(default_file_path)).unwrap();
        // load custom
        let _ = s.merge(File::from(custom_file_path));
        let result: Result<T, ConfigError> = s.try_into();
        result
    }

    pub fn read_config<T: DeserializeOwned>() -> Result<T, ConfigError> {
        ConfigManager::read_config_with_directory("config")
    }
}

#[cfg(test)]
mod test_config_manager {
    use crate::config_manage::config_manager::ConfigManager;
    use crate::config_manage::entity::config_entity::TestConfigEntity;

    #[test]
    fn basic() {
        let config: TestConfigEntity = ConfigManager::read_config().unwrap();
        dbg!(&config);
        assert_eq!(config.a, "aaa");
        assert_eq!(config.b.c, 1);
    }
}
