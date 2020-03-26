use crate::btc_on_eth::{
    errors::AppError,
    eth::{eth_database_utils::get_eth_private_key_from_db, eth_types::EthSignature},
    traits::DatabaseInterface,
    types::Result,
};

pub fn sign_message_with_eth_key<D, T>(db: &D, message: T) -> Result<EthSignature>
where
    D: DatabaseInterface,
    T: Into<String>,
{
    let message = message.into();

    info!("✔ Checking message is valid ASCII...");
    if !message.is_ascii() {
        return Err(AppError::Custom(
            "✘ Non-ASCII message passed. Only valid ASCII messages are supported.".to_string(),
        ));
    }

    let eth_private_key = get_eth_private_key_from_db(db)?;

    info!("✔ Signing message with eth key...");
    let signature = eth_private_key.sign_message_bytes(message.into_bytes())?;

    Ok(signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::{
        eth::{
            eth_database_utils::put_eth_private_key_in_db,
            eth_test_utils::get_sample_eth_private_key,
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_return_error_if_message_is_not_valid_ascii() {
        let db = get_test_database();
        let message = "Grüße, 🦀";
        assert!(sign_message_with_eth_key(&db, message).is_err());
    }

    #[test]

    fn should_sign_arbitrary_message() {
        let db = get_test_database();
        let eth_private_key = get_sample_eth_private_key();

        if let Err(e) = put_eth_private_key_in_db(&db, &eth_private_key) {
            panic!("Error putting eth private key in db: {}", e);
        }

        let message = "Arbitrary message";
        let valid_signature = [
            21, 167, 94, 225, 108, 8, 81, 23, 25, 12, 142, 251, 205, 52, 156, 213, 161, 168, 1, 79,
            228, 84, 149, 77, 14, 26, 128, 33, 14, 61, 91, 124, 26, 69, 95, 186, 93, 165, 20, 113,
            4, 94, 83, 226, 151, 246, 208, 131, 112, 153, 171, 166, 93, 77, 92, 91, 152, 174, 96,
            250, 66, 202, 68, 61, 0,
        ];

        // Arrays larger than 32 elements are not covered by std
        // thus require manual comparison
        assert!(
            sign_message_with_eth_key(&db, message)
                .unwrap()
                .iter()
                .zip(valid_signature.iter())
                .all(|(a, b)| a == b),
            "✘ Message signature is invalid!"
        );
    }
}
