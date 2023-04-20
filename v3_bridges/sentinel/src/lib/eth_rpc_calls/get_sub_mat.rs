use std::result::Result;

use common_eth::EthSubmissionMaterial;

use crate::{get_block, get_receipts, Endpoints, SentinelError};

pub async fn get_sub_mat(endpoints: &Endpoints, block_num: u64) -> Result<EthSubmissionMaterial, SentinelError> {
    let block = get_block(endpoints, block_num).await?;
    let ws_client = endpoints.get_rpc_client().await?;
    let receipts = get_receipts(&ws_client, &block.transactions).await?;
    Ok(EthSubmissionMaterial::default()
        .add_block(block)
        .and_then(|sub_mat| sub_mat.add_receipts(receipts))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_latest_block_num, test_utils::get_test_endpoints};

    #[tokio::test]
    async fn should_get_sub_mat() {
        let endpoints = get_test_endpoints().await;
        let block_num = get_latest_block_num(&endpoints).await.unwrap();
        let result = get_sub_mat(&endpoints, block_num).await;
        assert!(result.is_ok())
    }
}
