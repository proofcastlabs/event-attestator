use std::{result::Result, time::Duration};

use tokio::time::timeout;

use crate::{get_latest_block_num, Endpoints, SentinelError};

pub async fn check_endpoint(endpoints: &Endpoints, time_limit: u64) -> Result<(), SentinelError> {
    info!("Checking endpoint is working using a {time_limit}ms time limit...");
    match timeout(Duration::from_millis(time_limit), get_latest_block_num(endpoints)).await {
        Ok(_) => {
            info!("Endpoint check passed!");
            Ok(())
        },
        Err(e) => Err(SentinelError::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_test_endpoints;

    #[tokio::test]
    async fn working_endpoint_should_pass_endpoint_check() {
        let time_limit = 5000;
        let endpoints = get_test_endpoints().await;
        let result = check_endpoint(&endpoints, time_limit).await;
        assert!(result.is_ok());
    }
}
