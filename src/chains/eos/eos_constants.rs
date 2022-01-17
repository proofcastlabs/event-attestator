#[cfg(test)]
pub const EOS_ADDRESS_LENGTH_IN_BYTES: usize = 8;

pub const MEMO: &str = "";
pub const PRODUCER_REPS: u64 = 12;
pub const PUBLIC_KEY_SIZE: usize = 33;
pub const PEGIN_ACTION_NAME: &str = "pegin";
pub const PEGOUT_ACTION_NAME: &str = "pegout";
pub const REDEEM_ACTION_NAME: &str = "redeem";
pub const PUBLIC_KEY_CHECKSUM_SIZE: usize = 4;
pub const MAX_BYTES_FOR_EOS_USER_DATA: usize = 2000;
pub const EOS_SCHEDULE_DB_PREFIX: &str = "EOS_SCHEDULE_";
pub const EOS_ACCOUNT_PERMISSION_LEVEL: &str = "active";
pub const EOS_CORE_IS_INITIALIZED_JSON: &str = "{eos_core_initialized:true}";
pub const PUBLIC_KEY_WITH_CHECKSUM_SIZE: usize = PUBLIC_KEY_SIZE + PUBLIC_KEY_CHECKSUM_SIZE;
// NOTE: We use 59 minutes rather than 60 to give a little wiggle room for the clocks on the TEE devices.
pub const EOS_MAX_EXPIRATION_SECS: u32 = 3540;

create_db_keys_and_json!(
    "Eos";
    "PROCESSED_TX_IDS_KEY" => "eos-tx-ids",
    "EOS_INCREMERKLE_KEY" => "eos-incremerkle",
    "EOS_CHAIN_ID_DB_KEY" => "eos-chain-id-key",
    "EOS_TOKEN_SYMBOL_KEY" => "eos-token-ticker",
    "EOS_ACCOUNT_NAME_KEY" => "eos-account-name",
    "EOS_ACCOUNT_NONCE_KEY" => "eos-account-nonce",
    "EOS_SCHEDULE_LIST_KEY" => "eos-schedule-list",
    "EOS_PUBLIC_KEY_DB_KEY" => "eos-public-key-db-key",
    "EOS_PRIVATE_KEY_DB_KEY" => "eos-private-key-db-key",
    "EOS_PROTOCOL_FEATURES_KEY" => "eos-protocol-features",
    "EOS_LAST_SEEN_BLOCK_ID_KEY" => "eos-last-seen-block-id",
    "EOS_LAST_SEEN_BLOCK_NUM_KEY" => "eos-last-seen-block-num"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eos_database_keys_should_stay_consistent() {
        #[rustfmt::skip]
        let expected_result = EosDatabaseKeysJson {
            EOS_ACCOUNT_NAME_KEY:
                "8b9fd4b3e0a8263466a8fe52661124c424725ce71c62e0ac211f5ff022ada9a4".to_string(),
            EOS_ACCOUNT_NONCE_KEY:
                "165307417cab4f19b70e593876098df498c34ed3d38abedfc2a908eea4feaa82".to_string(),
            EOS_CHAIN_ID_DB_KEY:
                "cbd29a81186afbeb3af7e170ba5aad3b41426c3e81abc7562fa321f85426c6b3".to_string(),
            EOS_INCREMERKLE_KEY:
                "9a46679091d5f3b5f56e200451de1650c1bfbd3358c23293e1decfb5ab0d0fb1".to_string(),
            EOS_LAST_SEEN_BLOCK_ID_KEY:
                "5f38e7e4da08610c7d63bd371b28581a22f90ec9564079c4e2ce4322a0b4c8c3".to_string(),
            EOS_LAST_SEEN_BLOCK_NUM_KEY:
                "1ed3e38d13ec2aecc6ba97ca94ba1336a6cafeb105a8b45265dada291f05f369".to_string(),
            EOS_PRIVATE_KEY_DB_KEY:
                "d2d562ddd639ba2c7de122bc75f049a968ab759be57f66449c69d5f402723571".to_string(),
            EOS_PROTOCOL_FEATURES_KEY:
                "945786e2f66f06a6b4a14cab046919d0f51fdb4a73646104e898ffa73b44bc81".to_string(),
            EOS_PUBLIC_KEY_DB_KEY:
                "6307c57f8ebd700ef5d8db9cf8db34f7ee6cf4958e5a26db9466671e413a1324".to_string(),
            EOS_SCHEDULE_LIST_KEY:
                "d24e8320db81859d6e8ee6fa3ed7312155e489a2e8269c4ae8a2fa32a1ac5095".to_string(),
            EOS_TOKEN_SYMBOL_KEY:
                "71c8980fe3f6e8b3cdcbd4dce5f1a13af16e1980e3a7d4a570007c24d3691271".to_string(),
            PROCESSED_TX_IDS_KEY:
                "61b33e8588f6b6caa691d584efe8d3afadea0d16125650f85386b13e1f66e2e1".to_string(),
        };
        let result = EosDatabaseKeysJson::new();
        assert_eq!(result, expected_result);
    }
}
