use std::path::Path;

use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use structopt::StructOpt;
use tracing::info;

use crate::cmd_options::commandline_options::CommandlineOptions;

lazy_static! {
    static ref cmd_options:CommandlineOptions=CommandlineOptions::from_args();
}
pub struct ConfigManager {}

impl ConfigManager {
    pub fn read_config_with_directory<T: DeserializeOwned>(
        config_directory: &str,
    ) -> Result<T, ConfigError> {
        let mut s = Config::new();
        if let Some(external_config_directory) = &cmd_options.config_directory {
            info!("start to load from command line provided config directory {:?}", external_config_directory.as_os_str());
            let default_file_path = Path::new(external_config_directory).join(config_directory).join("default.toml");
            s.merge(File::from(default_file_path)).unwrap();
            let custom_file_path = Path::new(external_config_directory).join(config_directory).join("custom.toml");
            s.merge(File::from(custom_file_path));
        }

        let default_file_path = Path::new(config_directory).join("default.toml");
        // load default
        s.merge(File::from(default_file_path)).unwrap();
        let custom_file_path = Path::new(config_directory).join("custom.toml");
        // load custom
        s.merge(File::from(custom_file_path));
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
