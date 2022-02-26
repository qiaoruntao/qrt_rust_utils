use std::path::Path;

use cmd_lib::log::trace;
use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use structopt::StructOpt;
use tracing::info;

use crate::cmd_options::commandline_options::CommandlineOptions;

pub struct ConfigManager {}

impl ConfigManager {
    pub fn read_config_with_directory<T: DeserializeOwned>(
        config_directory: &str,
    ) -> Result<T, ConfigError> {
        let mut s = Config::new();

        let default_file_path = Path::new(config_directory).join("default.toml");
        // load default
        s.merge(File::from(default_file_path)).unwrap();
        let custom_file_path = Path::new(config_directory).join("custom.toml");
        // load custom
        match s.merge(File::from(custom_file_path)) {
            Ok(_) => {}
            Err(e) => {
                trace!("{}",e);
            }
        }
        let result: Result<T, ConfigError> = s.try_into();
        result
    }

    pub fn read_config<T: DeserializeOwned>() -> Result<T, ConfigError> {
        ConfigManager::read_config_with_directory("config")
    }
}

#[cfg(test)]
mod test_config_manager {
    use tracing::trace;

    use crate::config_manage::config_manager::ConfigManager;
    use crate::config_manage::entity::config_entity::TestConfigEntity;

    #[test]
    fn basic() {
        let config: TestConfigEntity = ConfigManager::read_config().unwrap();
        trace!("&config={:?}",&config);
        assert_eq!(config.a, "aaa");
        assert_eq!(config.b.c, 1);
    }
}
