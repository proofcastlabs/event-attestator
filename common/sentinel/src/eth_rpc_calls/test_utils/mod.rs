#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common_eth::EthSubmissionMaterial;

fn get_arbitrum_protocol_queue_operation_sub_mat() -> EthSubmissionMaterial {
    let path = "src/lib/eth_rpc_calls/test_utils/host-sub-mat-num-105419318.json";
    EthSubmissionMaterial::from_str(&read_to_string(path).unwrap()).unwrap()
}
