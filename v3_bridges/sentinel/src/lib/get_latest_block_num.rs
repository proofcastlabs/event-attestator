use std::result::Result;

use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::{constants::HEX_RADIX, endpoints::Error, Endpoints, SentinelError};

const GET_LATEST_BLOCK_NUM_RPC_CMD: &str = "eth_blockNumber";

pub async fn get_latest_block_num(endpoints: &Endpoints) -> Result<u64, SentinelError> {
    debug!("Getting latest block number via endpoint...");
    let client = endpoints.get_rpc_client().await?;
    let res: jsonrpsee::core::RpcResult<String> = client.request(GET_LATEST_BLOCK_NUM_RPC_CMD, rpc_params![]).await;
    match res {
        Err(_) => Err(SentinelError::Endpoint(Error::NoLatestBlock)),
        Ok(ref s) => Ok(u64::from_str_radix(&s.replace("0x", ""), HEX_RADIX)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_test_endpoints, get_test_ws_client};

    #[tokio::test]
    async fn should_get_latest_block_num() {
        let endpoints = get_test_endpoints().await;
        let result = get_latest_block_num(&endpoints).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }
}
