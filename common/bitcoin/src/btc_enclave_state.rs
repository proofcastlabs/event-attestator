use common::{safe_addresses::SAFE_BTC_ADDRESS_STR, traits::DatabaseInterface, types::Result};
use serde::{Deserialize, Serialize};

use crate::{
    btc_constants::{BTC_TAIL_LENGTH, MINIMUM_REQUIRED_SATOSHIS},
    btc_database_utils::BtcDbUtils,
    update_btc_linker_hash::get_linker_hash_or_genesis_hash as get_btc_linker_hash,
    utxo_manager::{get_total_number_of_utxos_from_db, get_total_utxo_balance_from_db, get_utxo_nonce_from_db},
};

#[derive(Serialize, Deserialize)]
pub struct BtcEnclaveState {
    btc_account_nonce: u64,
    btc_difficulty: u64,
    btc_network: String,
    btc_address: String,
    btc_utxo_nonce: u64,
    btc_tail_length: u64,
    btc_public_key: String,
    btc_sats_per_byte: u64,
    btc_linker_hash: String,
    btc_safe_address: String,
    btc_utxo_total_value: u64,
    btc_tail_block_number: u64,
    btc_number_of_utxos: usize,
    btc_canon_block_number: u64,
    btc_tail_block_hash: String,
    btc_canon_block_hash: String,
    btc_latest_block_number: u64,
    btc_anchor_block_number: u64,
    btc_canon_to_tip_length: u64,
    btc_latest_block_hash: String,
    btc_anchor_block_hash: String,
    btc_minimum_required_satoshis: u64,
}

impl BtcEnclaveState {
    // FIXME rm the database from this!
    pub fn new<D: DatabaseInterface>(db: &D, db_utils: &BtcDbUtils<D>) -> Result<Self> {
        info!("✔ Getting BTC enclave state...");
        let btc_tail_block = db_utils.get_btc_tail_block_from_db()?;
        let btc_canon_block = db_utils.get_btc_canon_block_from_db()?;
        let btc_anchor_block = db_utils.get_btc_anchor_block_from_db()?;
        let btc_latest_block = db_utils.get_btc_latest_block_from_db()?;
        let btc_public_key_hex = hex::encode(db_utils.get_btc_public_key_slice_from_db()?);
        Ok(Self {
            btc_tail_length: BTC_TAIL_LENGTH,
            btc_public_key: btc_public_key_hex,
            btc_address: db_utils.get_btc_address_from_db()?,
            btc_utxo_nonce: get_utxo_nonce_from_db(db)?,
            btc_tail_block_number: btc_tail_block.height,
            btc_sats_per_byte: db_utils.get_btc_fee_from_db()?,
            btc_canon_block_number: btc_canon_block.height,
            btc_safe_address: SAFE_BTC_ADDRESS_STR.to_string(),
            btc_latest_block_number: btc_latest_block.height,
            btc_difficulty: db_utils.get_btc_difficulty_from_db()?,
            btc_anchor_block_number: btc_anchor_block.height,
            btc_tail_block_hash: btc_tail_block.id.to_string(),
            btc_canon_block_hash: btc_canon_block.id.to_string(),
            btc_account_nonce: db_utils.get_btc_account_nonce_from_db()?,
            btc_linker_hash: get_btc_linker_hash(db_utils)?.to_string(),
            btc_network: db_utils.get_btc_network_from_db()?.to_string(),
            btc_latest_block_hash: btc_latest_block.id.to_string(),
            btc_anchor_block_hash: btc_anchor_block.id.to_string(),
            btc_minimum_required_satoshis: MINIMUM_REQUIRED_SATOSHIS,
            btc_utxo_total_value: get_total_utxo_balance_from_db(db)?,
            btc_number_of_utxos: get_total_number_of_utxos_from_db(db),
            btc_canon_to_tip_length: db_utils.get_btc_canon_to_tip_length_from_db()?,
        })
    }
}
