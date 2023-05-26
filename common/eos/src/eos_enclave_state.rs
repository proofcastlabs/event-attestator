use common::{
    constants::FIELD_NOT_SET_MSG,
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryJson},
    traits::DatabaseInterface,
    types::Result,
};
use common_safe_addresses::SAFE_EOS_ADDRESS_STR;
use serde::{Deserialize, Serialize};

use crate::{
    eos_database_utils::EosDbUtils,
    eos_global_sequences::ProcessedGlobalSequences,
    eos_types::EosKnownSchedulesJsons,
    protocol_features::EnabledFeatures,
};

#[derive(Serialize, Deserialize)]
pub struct EosEnclaveState {
    eos_account_name: String,
    eos_chain_id: String,
    eos_public_key: String,
    eos_safe_address: String,
    eos_signature_nonce: u64,
    eos_last_seen_block_num: u64,
    eos_last_seen_block_id: String,
    eos_known_schedules: EosKnownSchedulesJsons,
    eos_enabled_protocol_features: EnabledFeatures,
    eos_eth_token_dictionary: EosEthTokenDictionaryJson,
    processed_global_sequences: ProcessedGlobalSequences,
}

impl EosEnclaveState {
    pub fn new<D: DatabaseInterface>(eos_db_utils: &EosDbUtils<D>) -> Result<Self> {
        Self::new_maybe_with_account_name(eos_db_utils, true)
    }

    pub fn new_without_account_name<D: DatabaseInterface>(eos_db_utils: &EosDbUtils<D>) -> Result<Self> {
        Self::new_maybe_with_account_name(eos_db_utils, false)
    }

    pub fn new_maybe_with_account_name<D: DatabaseInterface>(
        eos_db_utils: &EosDbUtils<D>,
        include_account_name: bool,
    ) -> Result<Self> {
        info!("✔ Getting EOS enclave state...");
        Ok(EosEnclaveState {
            eos_safe_address: SAFE_EOS_ADDRESS_STR.to_string(),
            eos_chain_id: eos_db_utils.get_eos_chain_id_from_db()?.to_hex(),
            eos_signature_nonce: eos_db_utils.get_eos_account_nonce_from_db()?,
            eos_last_seen_block_num: eos_db_utils.get_latest_eos_block_number()?,
            eos_public_key: eos_db_utils.get_eos_public_key_from_db()?.to_string(),
            eos_last_seen_block_id: eos_db_utils.get_eos_last_seen_block_id_from_db()?.to_string(),
            processed_global_sequences: ProcessedGlobalSequences::get_from_db(eos_db_utils.get_db())?,
            eos_enabled_protocol_features: eos_db_utils.get_eos_enabled_protocol_features_from_db()?,
            eos_eth_token_dictionary: EosEthTokenDictionary::get_from_db(eos_db_utils.get_db())?.to_json()?,
            eos_known_schedules: EosKnownSchedulesJsons::from_schedules(
                eos_db_utils.get_eos_known_schedules_from_db()?,
            ),
            eos_account_name: if include_account_name {
                eos_db_utils.get_eos_account_name_string_from_db()?
            } else {
                FIELD_NOT_SET_MSG.to_string()
            },
        })
    }
}
