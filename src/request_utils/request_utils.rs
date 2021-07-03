use std::time::Duration;

use reqwest::{Client, Proxy};
use reqwest::header::HeaderMap;

pub struct RequestUtils {}

impl RequestUtils {
    pub fn get_client(timeout: Option<Duration>, proxy_address: Option<String>) -> Client {
        let mut header_map = HeaderMap::new();
        header_map.append("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.93 Safari/537.36".parse().unwrap());
        header_map.append("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".parse().unwrap());
        // header_map.append("Accept-Encoding", "gzip, deflate".parse().unwrap());
        header_map.append("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
        let mut builder = reqwest::Client::builder()
            .default_headers(header_map)
            // .proxy(Proxy::all("socks5://127.0.0.1:1092").unwrap())
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(5));
        if let Some(timeout) = timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(proxy_address) = proxy_address {
            builder = builder.proxy(Proxy::all(proxy_address).unwrap());
        }
        let client = builder.build().unwrap();
        client
    }
}
