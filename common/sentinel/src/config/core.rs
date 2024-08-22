use std::{net::SocketAddr, str::FromStr};

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Getters, Deserialize, Eq, PartialEq, Serialize)]
pub struct SentinelCoreConfig {
    timeout: u64,
    rpc_server_address: SocketAddr,
}

impl Default for SentinelCoreConfig {
    fn default() -> Self {
        Self {
            timeout: u64::default(),
            rpc_server_address: SocketAddr::from_str("127.0.0.1:3030").expect("this not to fail"),
        }
    }
}
