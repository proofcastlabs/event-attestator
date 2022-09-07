use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        eos::eos_crypto::eos_transaction::EosSignedTransaction,
        eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    },
    int_on_eos::int::eos_tx_info::IntOnEosEosTxInfo,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosTxInfo {
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
    pub destination_chain_id: String,
    pub native_token_address: String,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

impl EosTxInfo {
    pub fn new(
        eos_tx: &EosSignedTransaction,
        tx_info: &IntOnEosEosTxInfo,
        eos_account_nonce: u64,
        eos_latest_block_number: u64,
    ) -> Result<EosTxInfo> {
        Ok(EosTxInfo {
            broadcast: false,
            eos_account_nonce,
            eos_latest_block_number,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            eos_tx_amount: eos_tx.amount.to_string(),
            eos_tx_signature: eos_tx.signature.clone(),
            eos_tx_recipient: eos_tx.recipient.clone(),
            eos_serialized_tx: eos_tx.transaction.clone(),
            int_tx_amount: tx_info.token_amount.to_string(),
            _id: format!("pint-on-eos-eos-{}", eos_account_nonce),
            host_token_address: tx_info.eos_token_address.to_string(),
            destination_chain_id: tx_info.destination_chain_id.to_hex()?,
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender)),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash)),
            native_token_address: format!("0x{}", hex::encode(tx_info.eth_token_address)),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntOnEosEosOutput {
    pub eth_latest_block_number: u64,
    pub eos_signed_transactions: Vec<EosTxInfo>,
}

pub fn get_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting `IntOnEos` EOS output json...");
    Ok(serde_json::to_string(&IntOnEosEosOutput {
        eth_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_u64(),
        eos_signed_transactions: match state.eos_transactions {
            None => vec![],
            Some(ref eos_txs) => {
                let num_txs = eos_txs.len() as u64;
                let tx_infos = state.int_on_eos_eos_tx_infos.clone();
                let eos_account_nonce = state.eos_db_utils.get_eos_account_nonce_from_db()?;
                let eos_latest_block_num = state.eos_db_utils.get_latest_eos_block_number()?;
                let start_nonce = if num_txs > eos_account_nonce {
                    return Err("EOS account nonce has not been incremented correctly!".into());
                } else {
                    eos_account_nonce - num_txs
                };
                if tx_infos.len() as u64 != num_txs {
                    return Err("Number of EOS txs does not match number of tx infos!".into());
                };
                eos_txs
                    .iter()
                    .enumerate()
                    .map(|(i, eos_tx)| {
                        EosTxInfo::new(eos_tx, &tx_infos[i], start_nonce + i as u64, eos_latest_block_num)
                    })
                    .collect::<Result<Vec<EosTxInfo>>>()?
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
impl FromStr for IntOnEosEosOutput {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
impl FromStr for EosTxInfo {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}
