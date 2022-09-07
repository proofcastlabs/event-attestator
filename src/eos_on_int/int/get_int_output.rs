use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        eos::{eos_chain_id::EosChainId, eos_crypto::eos_transaction::EosSignedTransaction},
        eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    },
    eos_on_int::int::eos_tx_info::EosOnIntEosTxInfo,
    metadata::ToMetadataChainId,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub eos_tx_amount: String,
    pub int_tx_amount: String,
    pub eos_account_nonce: u64,
    pub eos_tx_recipient: String,
    pub eos_tx_signature: String,
    pub witnessed_timestamp: u64,
    pub eos_serialized_tx: String,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub eos_latest_block_number: u64,
    pub native_token_address: String,
    pub destination_chain_id: String,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

impl TxInfo {
    pub fn new(
        eos_tx: &EosSignedTransaction,
        tx_info: &EosOnIntEosTxInfo,
        eos_account_nonce: u64,
        eos_latest_block_number: u64,
        eos_chain_id: &EosChainId,
    ) -> Result<TxInfo> {
        Ok(TxInfo {
            broadcast: false,
            eos_account_nonce,
            eos_latest_block_number,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            eos_tx_signature: eos_tx.signature.clone(),
            eos_tx_recipient: eos_tx.recipient.clone(),
            eos_serialized_tx: eos_tx.transaction.clone(),
            int_tx_amount: tx_info.token_amount.to_string(),
            eos_tx_amount: tx_info.eos_asset_amount.clone(),
            _id: format!("peos-on-int-eos-{}", eos_account_nonce),
            native_token_address: tx_info.eos_token_address.to_string(),
            destination_chain_id: eos_chain_id.to_metadata_chain_id().to_hex()?,
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender)),
            host_token_address: format!("0x{}", hex::encode(tx_info.int_token_address)),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntOutput {
    pub int_latest_block_number: u64,
    pub eos_signed_transactions: Vec<TxInfo>,
}

pub fn get_int_output<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting `eos-on-int` INT submission output json...");
    Ok(serde_json::to_string(&IntOutput {
        int_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_u64(),
        eos_signed_transactions: match state.eos_transactions {
            None => vec![],
            Some(ref eos_txs) => {
                let number_of_txs = eos_txs.len() as u64;
                let eos_account_nonce = state.eos_db_utils.get_eos_account_nonce_from_db()?;
                let start_nonce = if number_of_txs > eos_account_nonce {
                    return Err("EOS account nonce has not been incremented correctly!".into());
                } else {
                    eos_account_nonce - number_of_txs
                };
                eos_txs
                    .iter()
                    .enumerate()
                    .map(|(i, eos_tx)| {
                        TxInfo::new(
                            eos_tx,
                            &state.eos_on_int_eos_tx_infos[i],
                            start_nonce + i as u64,
                            state.eos_db_utils.get_latest_eos_block_number()?,
                            &state.eos_db_utils.get_eos_chain_id_from_db()?,
                        )
                    })
                    .collect::<Result<Vec<TxInfo>>>()?
            },
        },
    })?)
}

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
use serde_json;

#[cfg(test)]
use crate::errors::AppError;

#[cfg(test)]
impl FromStr for IntOutput {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
impl FromStr for TxInfo {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}
