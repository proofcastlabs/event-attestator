use ethereum_types::{
    U256,
    H160,
    Address,
};
use crate::{
    types::Result,
    chains::eth::{
        eth_types::{
            EthReceipt,
            EthReceipts,
            EthereumLogs,
            EthReceiptJson,
        },
    },
    btc_on_eth::utils::{
        convert_hex_to_h256,
        convert_hex_to_address,
        convert_json_value_to_string,
    },
};

pub fn parse_eth_receipt_jsons(eth_receipts_jsons: Vec<EthReceiptJson>) -> Result<EthReceipts> { // TODO Make a type so we can impl on it!
    trace!("✔ Parsing ETH receipt JSON...");
    eth_receipts_jsons.iter().cloned().map(|receipt_json| EthReceipt::from_json(&receipt_json)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        get_expected_receipt,
        SAMPLE_RECEIPT_INDEX,
        get_sample_eth_block_and_receipts_json,
    };

    #[test]
    fn should_parse_eth_receipt_jsons() {
        let eth_json = get_sample_eth_block_and_receipts_json().unwrap();
        if parse_eth_receipt_jsons(eth_json.receipts).is_err() {
            panic!("Should have generated receipts correctly!")
        }
    }
}
