pub use serde_json::{
    json,
    Value as JsonValue,
};
use crate::{
    types::{
        Bytes,
        Result,
    },
    chains::eth::{
        eth_types::EthSignature,
        eth_block_and_receipts::EthBlockAndReceipts,
    },
};

// TODO rm this in favour of impl!
pub fn decode_eth_block_and_receipts_from_json_bytes(block_and_receipt_bytes: Bytes) -> Result<EthBlockAndReceipts> {
    EthBlockAndReceipts::from_json(&serde_json::from_slice(&block_and_receipt_bytes)?)
}

pub fn encode_eth_signed_message_as_json(
    message: &str,
    signature: &EthSignature
) -> Result<JsonValue> {
    info!("✔ Encoding eth signed message as json...");
    Ok(json!({
        "message": message,
        "signature": format!("0x{}", hex::encode(&signature[..]))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::get_sample_eth_block_and_receipts;

    #[test]
    fn should_decode_block_and_recipts_json_correctly() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let bytes = block_and_receipts.to_bytes().unwrap();
        let result = decode_eth_block_and_receipts_from_json_bytes(bytes)
            .unwrap();
        assert_eq!(result.block, block_and_receipts.block);
        block_and_receipts
            .receipts
            .iter()
            .enumerate()
            .map(|(i, receipt)| assert_eq!(receipt, &result.receipts[i]))
            .for_each(drop);
    }

    #[test]
    fn should_encode_eth_signed_message_as_json() {
        let valid_json = json!({
            "message": "Arbitrary message",
            "signature": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        });

        assert_eq!(
            encode_eth_signed_message_as_json("Arbitrary message", &[0u8; 65]).unwrap(),
            valid_json,
            "✘ Message signature json is invalid!"
        )
    }
}
