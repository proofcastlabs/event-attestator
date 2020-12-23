use crate::{
    chains::eth::{eth_database_utils::get_eth_private_key_from_db, eth_types::EthSignature},
    traits::DatabaseInterface,
    types::Result,
};
use serde_json::{json, Value as JsonValue};

fn encode_eth_signed_message_as_json(message: &str, signature: &EthSignature) -> Result<JsonValue> {
    info!("✔ Encoding eth signed message as json...");
    Ok(json!({"message": message, "signature": format!("0x{}", hex::encode(&signature[..]))}))
}

pub fn sign_ascii_msg_with_eth_key_with_no_prefix<D: DatabaseInterface>(db: &D, message: &str) -> Result<String> {
    info!("✔ Checking message is valid ASCII...");
    if !message.is_ascii() {
        return Err("✘ Non-ASCII message passed. Only valid ASCII messages are supported.".into());
    }
    get_eth_private_key_from_db(db)
        .and_then(|key| key.sign_message_bytes(message.as_bytes()))
        .and_then(|signature| encode_eth_signed_message_as_json(&message, &signature))
        .map(|json| json.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        btc_on_eth::eth::eth_test_utils::get_sample_eth_private_key,
        chains::eth::eth_database_utils::put_eth_private_key_in_db,
        test_utils::get_test_database,
    };

    #[test]
    fn should_return_error_if_message_is_not_valid_ascii() {
        let db = get_test_database();
        let message = "Grüße, 🦀";
        assert!(sign_ascii_msg_with_eth_key_with_no_prefix(&db, message).is_err());
    }

    #[test]
    fn should_sign_ascii_msg_with_eth_key_with_no_prefix() {
        let db = get_test_database();
        let eth_private_key = get_sample_eth_private_key();
        put_eth_private_key_in_db(&db, &eth_private_key).unwrap();
        let message = "Arbitrary message";
        let expected_result = json!({
            "message": "Arbitrary message",
            "signature": "0x15a75ee16c085117190c8efbcd349cd5a1a8014fe454954d0e1a80210e3d5b7c1a455fba5da51471045e53e297f6d0837099aba65d4d5c5b98ae60fa42ca443d00"
        }).to_string();
        let result = sign_ascii_msg_with_eth_key_with_no_prefix(&db, message).unwrap();
        assert_eq!(result, expected_result, "✘ Message signature is invalid!")
    }

    #[test]
    fn should_encode_eth_signed_message_as_json() {
        let expected_result = json!({
            "message": "Arbitrary message",
            "signature": "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        });
        let result = encode_eth_signed_message_as_json("Arbitrary message", &[0u8; 65]).unwrap();
        assert_eq!(result, expected_result, "✘ Message signature json is invalid!")
    }
}
