

use tracing::error;

use crate::config_manage::config_manager::ConfigManager;
use crate::proxy_pool::entity::proxy_pool_config::ProxyPoolConfig;
use crate::proxy_pool::entity::proxy_pool_get_response::ProxyPoolGetResponse;
use crate::request_utils::request_utils::RequestUtils;

pub struct ProxyPoolManager {}

impl ProxyPoolManager {
    pub async fn fetch_proxy() -> Option<ProxyPoolGetResponse> {
        let config: ProxyPoolConfig =
            ConfigManager::read_config_with_directory("config/proxy_pool").unwrap();
        let client = RequestUtils::build_client(None, None);

        let response = match client.get(format!("{}/get/?type=https", &config.address)).send().await {
            Ok(response) => { response }
            Err(e) => {
                error!("cannot fetch proxy info {:?}",e);
                return None;
            }
        };
        let content = response.text().await.unwrap();
        match serde_json::from_str(content.as_str()) {
            Ok(val) => {
                Some(val)
            }
            Err(e) => {
                error!("cannot deserialize proxy info, content={:?}, error={}", &content,&e);
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::logger::logger::{Logger, LoggerConfig};
    use crate::proxy_pool::proxy_pool_manager::ProxyPoolManager;

    #[tokio::test]
    async fn basic_test() {
        Logger::init_logger(&LoggerConfig::default());
        let proxy = ProxyPoolManager::fetch_proxy().await;
        println!("{:?}", proxy);
    }
}