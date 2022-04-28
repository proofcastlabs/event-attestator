use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        eos::eos_state::EosState,
        eth::{
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::EthDbUtilsExt,
            eth_traits::EthTxInfoCompatible,
        },
    },
    int_on_eos::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos},
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EosOutput {
    pub eos_latest_block_number: u64,
    pub int_signed_transactions: Vec<TxInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub int_tx_hash: String,
    pub int_tx_amount: String,
    pub eos_tx_amount: String,
    pub int_account_nonce: u64,
    pub int_tx_recipient: String,
    pub witnessed_timestamp: u64,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub native_token_address: String,
    pub int_signed_tx: Option<String>,
    pub int_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

impl TxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        tx_info: &IntOnEosIntTxInfo,
        nonce: u64,
        int_latest_block_number: usize,
    ) -> Result<TxInfo> {
        Ok(TxInfo {
            int_latest_block_number,
            broadcast: false,
            broadcast_tx_hash: None,
            int_account_nonce: nonce,
            broadcast_timestamp: None,
            int_signed_tx: tx.eth_tx_hex(),
            _id: format!("pint-on-eos-int-{}", nonce),
            int_tx_amount: tx_info.amount.to_string(),
            int_tx_hash: format!("0x{}", tx.get_tx_hash()),
            eos_tx_amount: tx_info.eos_tx_amount.to_string(),
            originating_address: tx_info.origin_address.to_string(),
            host_token_address: tx_info.eos_token_address.to_string(),
            originating_tx_hash: tx_info.originating_tx_id.to_string(),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            int_tx_recipient: format!("0x{}", hex::encode(tx_info.destination_address.as_bytes())),
            native_token_address: format!("0x{}", hex::encode(&tx_info.int_token_address)),
        })
    }
}

pub fn get_tx_infos_from_signed_txs(
    txs: &[EthTransaction],
    redeem_info: &IntOnEosIntTxInfos,
    int_account_nonce: u64,
    any_sender_nonce: u64,
    int_latest_block_number: usize,
) -> Result<Vec<TxInfo>> {
    info!("✔ Getting output from INT txs...");
    let number_of_txs = txs.len() as u64;
    let start_nonce = if number_of_txs > int_account_nonce {
        return Err("INT account nonce has not been incremented correctly!".into());
    } else {
        int_account_nonce - number_of_txs
    };
    txs.iter()
        .enumerate()
        .map(|(i, tx)| TxInfo::new(tx, &redeem_info[i], start_nonce, int_latest_block_number))
        .collect::<Result<Vec<TxInfo>>>()
}

pub fn get_eos_output<D: DatabaseInterface>(state: EosState<D>) -> Result<String> {
    info!("✔ Getting EOS output json...");
    let int_signed_txs = state.eth_signed_txs.clone();
    let output = serde_json::to_string(&EosOutput {
        eos_latest_block_number: state.eos_db_utils.get_latest_eos_block_number()?,
        int_signed_transactions: if int_signed_txs.is_empty() {
            vec![]
        } else {
            get_tx_infos_from_signed_txs(
                &int_signed_txs,
                &state.int_on_eos_int_tx_infos,
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                state.eth_db_utils.get_any_sender_nonce_from_db()?,
                state.eth_db_utils.get_latest_eth_block_number()?,
            )?
        },
    })?;
    info!("✔ EOS output: {}", output);
    Ok(output)
}
