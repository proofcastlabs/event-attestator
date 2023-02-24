use std::str::FromStr;

use common::{
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    database_utils::{get_string_from_db, get_u64_from_db, put_string_in_db, put_u64_in_db},
    traits::DatabaseInterface,
    types::{Bytes, Result},
};
use common_chain_ids::EosChainId;
use eos_chain::{AccountName as EosAccountName, Checksum256};

use crate::{
    eos_crypto::eos_public_key::EosPublicKey,
    eos_merkle_utils::{Incremerkle, IncremerkleJson},
    eos_producer_schedule::EosProducerScheduleV2,
    eos_types::EosKnownSchedules,
    eos_utils::{convert_hex_to_checksum256, get_eos_schedule_db_key},
    protocol_features::EnabledFeatures,
};

create_db_utils_with_getters!(
    "Eos";
    "_PROCESSED_TX_IDS_KEY" => "eos-tx-ids",
    "_INCREMERKLE_KEY" => "eos-incremerkle",
    "_CHAIN_ID_DB_KEY" => "eos-chain-id-key",
    "_TOKEN_SYMBOL_KEY" => "eos-token-ticker",
    "_ACCOUNT_NAME_KEY" => "eos-account-name",
    "_ACCOUNT_NONCE_KEY" => "eos-account-nonce",
    "_SCHEDULE_LIST_KEY" => "eos-schedule-list",
    "_PUBLIC_KEY_DB_KEY" => "eos-public-key-db-key",
    "_PRIVATE_KEY_DB_KEY" => "eos-private-key-db-key",
    "_PROTOCOL_FEATURES_KEY" => "eos-protocol-features",
    "_LAST_SEEN_BLOCK_ID_KEY" => "eos-last-seen-block-id",
    "_LAST_SEEN_BLOCK_NUM_KEY" => "eos-last-seen-block-num"
);

impl<'a, D: DatabaseInterface> EosDbUtils<'a, D> {
    pub fn put_eos_public_key_in_db(&self, public_key: &EosPublicKey) -> Result<()> {
        debug!("✔ Putting EOS public key in db...");
        self.db.put(
            self.get_eos_public_key_db_key(),
            public_key.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_eos_public_key_from_db(&self) -> Result<EosPublicKey> {
        debug!("✔ Getting EOS public key from db...");
        self.get_db()
            .get(self.get_eos_public_key_db_key(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| EosPublicKey::from_bytes(&bytes))
    }

    pub fn put_eos_enabled_protocol_features_in_db(&self, protocol_features: &EnabledFeatures) -> Result<()> {
        debug!("✔ Putting EOS enabled protocol features in db...");
        self.get_db().put(
            self.get_eos_protocol_features_key(),
            protocol_features.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_eos_enabled_protocol_features_from_db(&self) -> Result<EnabledFeatures> {
        debug!("✔ Getting EOS enabled protocol features from db...");
        match self
            .get_db()
            .get(self.get_eos_protocol_features_key(), MIN_DATA_SENSITIVITY_LEVEL)
        {
            Ok(bytes) => EnabledFeatures::from_bytes(&bytes),
            Err(_) => {
                info!("✔ No features found in db! Initting empty features...");
                Ok(EnabledFeatures::init())
            },
        }
    }

    pub fn put_eos_last_seen_block_num_in_db(&self, num: u64) -> Result<()> {
        debug!("✔ Putting EOS last seen block num in db...");
        put_u64_in_db(self.get_db(), &self.get_eos_last_seen_block_num_key(), num)
    }

    pub fn get_latest_eos_block_number(&self) -> Result<u64> {
        debug!("✔ Getting EOS latest block number from db...");
        get_u64_from_db(self.get_db(), &EOS_LAST_SEEN_BLOCK_NUM_KEY.to_vec())
    }

    pub fn put_eos_last_seen_block_id_in_db(&self, latest_block_id: &Checksum256) -> Result<()> {
        let block_id_string = latest_block_id.to_string();
        debug!("✔ Putting EOS latest block ID {} in db...", block_id_string);
        put_string_in_db(self.get_db(), &self.get_eos_last_seen_block_id_key(), &block_id_string)
    }

    pub fn get_eos_last_seen_block_id_from_db(&self) -> Result<Checksum256> {
        debug!("✔ Getting EOS last seen block ID from db...");
        get_string_from_db(self.get_db(), &self.get_eos_last_seen_block_id_key()).and_then(convert_hex_to_checksum256)
    }

    pub fn put_incremerkle_in_db(&self, incremerkle: &Incremerkle) -> Result<()> {
        debug!("✔ Putting EOS incremerkle in db...");
        self.get_db().put(
            self.get_eos_incremerkle_key(),
            serde_json::to_vec(&incremerkle.to_json())?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn get_incremerkle_from_db(&self) -> Result<Incremerkle> {
        debug!("✔ Getting EOS incremerkle from db...");
        self.get_db()
            .get(self.get_eos_incremerkle_key(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(serde_json::from_slice(&bytes)?))
            .and_then(|json: IncremerkleJson| json.to_incremerkle())
    }

    pub fn get_eos_known_schedules_from_db(&self) -> Result<EosKnownSchedules> {
        debug!("✔ Getting EOS known schedules from db...");
        self.get_db()
            .get(self.get_eos_schedule_list_key(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(serde_json::from_slice(&bytes)?))
    }

    pub fn put_eos_known_schedules_in_db(&self, eos_known_schedules: &EosKnownSchedules) -> Result<()> {
        debug!("✔ Putting EOS known schedules in db: {}", &eos_known_schedules);
        self.get_db().put(
            self.get_eos_schedule_list_key(),
            serde_json::to_vec(eos_known_schedules)?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    pub fn put_eos_schedule_in_db(&self, schedule: &EosProducerScheduleV2) -> Result<()> {
        let db_key = get_eos_schedule_db_key(schedule.version);
        match self.get_db().get(db_key.clone(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(_) => {
                debug!("✘ EOS schedule {} already in db!", &schedule.version);
                Ok(())
            },
            Err(_) => {
                debug!("✔ Putting EOS schedule in db: {:?}", schedule);
                put_string_in_db(self.get_db(), &db_key, &serde_json::to_string(schedule)?)
                    .and_then(|_| self.get_eos_known_schedules_from_db())
                    .map(|scheds| scheds.add_new(schedule.version))
                    .and_then(|scheds| self.put_eos_known_schedules_in_db(&scheds))
            },
        }
    }

    pub fn get_eos_schedule_from_db(&self, version: u32) -> Result<EosProducerScheduleV2> {
        debug!("✔ Getting EOS schedule from db...");
        match get_string_from_db(self.get_db(), &get_eos_schedule_db_key(version)) {
            Ok(json) => EosProducerScheduleV2::from_json(&json),
            Err(_) => Err(format!("✘ Core does not have EOS schedule version: {}", version).into()),
        }
    }

    pub fn get_eos_account_nonce_from_db(&self) -> Result<u64> {
        debug!("✔ Getting EOS account nonce from db...");
        get_u64_from_db(self.get_db(), &self.get_eos_account_nonce_key())
    }

    pub fn put_eos_account_nonce_in_db(&self, new_nonce: u64) -> Result<()> {
        debug!("✔ Putting EOS account nonce in db...");
        put_u64_in_db(self.get_db(), &self.get_eos_account_nonce_key(), new_nonce)
    }

    pub fn put_eos_token_symbol_in_db(&self, name: &str) -> Result<()> {
        debug!("✔ Putting EOS token symbol in db...");
        put_string_in_db(self.get_db(), &self.get_eos_token_symbol_key(), name)
    }

    pub fn get_eos_token_symbol_from_db(&self) -> Result<String> {
        debug!("✔ Getting EOS token symbol from db...");
        get_string_from_db(self.get_db(), &self.get_eos_token_symbol_key())
    }

    pub fn put_eos_account_name_in_db(&self, name: &str) -> Result<()> {
        debug!("✔ Putting EOS account name in db...");
        put_string_in_db(self.get_db(), &self.get_eos_account_name_key(), name)
    }

    pub fn get_eos_account_name_string_from_db(&self) -> Result<String> {
        debug!("✔ Getting EOS account name string from db...");
        get_string_from_db(self.get_db(), &self.get_eos_account_name_key())
    }

    pub fn get_eos_account_name_from_db(&self) -> Result<EosAccountName> {
        debug!("✔ Getting EOS account name from db...");
        match self.get_eos_account_name_string_from_db() {
            Err(_) => Err("No EOS account name in DB! Did you forget to set it?".into()),
            Ok(ref s) => Ok(EosAccountName::from_str(s)?),
        }
    }

    pub fn put_eos_chain_id_in_db(&self, chain_id: &EosChainId) -> Result<()> {
        debug!("✔ Putting EOS chain ID in db...");
        put_string_in_db(self.get_db(), &self.get_eos_chain_id_db_key(), &chain_id.to_hex())
    }

    pub fn get_eos_chain_id_from_db(&self) -> Result<EosChainId> {
        info!("✔ Getting EOS chain ID from db...");
        EosChainId::from_str(&get_string_from_db(self.get_db(), &self.get_eos_chain_id_db_key())?)
    }
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::eos_test_utils::get_sample_eos_public_key;

    #[test]
    fn should_put_and_get_eos_public_key_in_db_correctly() {
        let db = get_test_database();
        let db_utils = EosDbUtils::new(&db);
        let key = get_sample_eos_public_key();
        db_utils.put_eos_public_key_in_db(&key).unwrap();
        let result = db_utils.get_eos_public_key_from_db().unwrap();
        assert_eq!(key, result);
    }

    #[test]
    fn eos_database_keys_should_stay_consistent() {
        #[rustfmt::skip]
        let expected_result = EosDatabaseKeysJson {
            HOST_CORE_IS_INITIALIZED_DB_KEY:
                "0271c9a9e186967bbd36c4eb36f47a94d3771ace3879f1bbf202842c89942999".to_string(),
            NATIVE_CORE_IS_INITIALIZED_DB_KEY:
                "afa4de60dc3ab1362c4b4acf9536393ece435e3e7951363c6ea87182939271f3".to_string(),
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
            EOS_PROCESSED_TX_IDS_KEY:
                "61b33e8588f6b6caa691d584efe8d3afadea0d16125650f85386b13e1f66e2e1".to_string(),
        };
        let result = EosDatabaseKeysJson::new();
        assert_eq!(result, expected_result);
    }
}
