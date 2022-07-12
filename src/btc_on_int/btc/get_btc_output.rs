#[cfg(test)]
use std::str::FromStr;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::Value as JsonValue;

#[cfg(test)]
use crate::errors::AppError;
use crate::{
    btc_on_int::btc::int_tx_info::BtcOnIntIntTxInfo,
    chains::{
        btc::btc_state::BtcState,
        eth::{
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::EthDbUtilsExt,
            eth_traits::EthTxInfoCompatible,
        },
    },
    traits::DatabaseInterface,
    types::{NoneError, Result},
    utils::get_unix_timestamp,
};

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Constructor)]
pub struct BtcOutput {
    pub btc_latest_block_number: u64,
    pub int_signed_transactions: Vec<IntTxInfo>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct IntTxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub int_tx_hash: String,
    pub int_signed_tx: String,
    pub int_tx_amount: String,
    pub int_account_nonce: u64,
    pub witnessed_timestamp: u64,
    pub int_tx_recipient: String,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub destination_chain_id: String,
    pub int_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

#[cfg(test)]
impl FromStr for IntTxInfo {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
impl FromStr for BtcOutput {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        #[derive(Serialize, Deserialize)]
        struct Interim {
            btc_latest_block_number: u64,
            int_signed_transactions: Vec<JsonValue>,
        }
        let interim = serde_json::from_str::<Interim>(s)?;
        let tx_infos = interim
            .int_signed_transactions
            .iter()
            .map(|json| IntTxInfo::from_str(&json.to_string()))
            .collect::<Result<Vec<IntTxInfo>>>()?;
        Ok(Self {
            btc_latest_block_number: interim.btc_latest_block_number,
            int_signed_transactions: tx_infos,
        })
    }
}

impl IntTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        int_tx_info: &BtcOnIntIntTxInfo,
        nonce: u64,
        int_latest_block_number: usize,
    ) -> Result<IntTxInfo> {
        Ok(IntTxInfo {
            broadcast: false,
            int_latest_block_number,
            broadcast_tx_hash: None,
            int_account_nonce: nonce,
            broadcast_timestamp: None,
            _id: format!("pbtc-on-int-int-{}", nonce),
            witnessed_timestamp: get_unix_timestamp()?,
            int_tx_hash: format!("0x{}", tx.get_tx_hash()),
            int_tx_amount: int_tx_info.host_token_amount.to_string(),
            host_token_address: int_tx_info.int_token_address.clone(),
            int_tx_recipient: int_tx_info.destination_address.clone(),
            originating_address: int_tx_info.originating_tx_address.clone(),
            originating_tx_hash: int_tx_info.originating_tx_hash.to_string(),
            int_signed_tx: tx.eth_tx_hex().ok_or(NoneError("No tx in tx info!"))?,
            destination_chain_id: format!("0x{}", hex::encode(&int_tx_info.destination_chain_id)),
        })
    }
}

pub fn get_eth_signed_tx_info_from_eth_txs(
    int_txs: &[EthTransaction],
    int_tx_infos: &[BtcOnIntIntTxInfo],
    int_account_nonce: u64,
    int_latest_block_number: usize,
) -> Result<Vec<IntTxInfo>> {
    info!("✔ Getting INT tx info from INT txs...");
    let number_of_txs = int_txs.len() as u64;
    let start_nonce = if number_of_txs > int_account_nonce {
        return Err("INT account nonce has not been incremented correctly!".into());
    } else {
        int_account_nonce - number_of_txs
    };
    int_txs
        .iter()
        .enumerate()
        .map(|(i, tx)| IntTxInfo::new(tx, &int_tx_infos[i], start_nonce + i as u64, int_latest_block_number))
        .collect::<Result<Vec<IntTxInfo>>>()
}

pub fn get_btc_output_and_put_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Getting BTC output json and putting in state...");
    let signed_txs = state.eth_signed_txs.clone();
    let output = BtcOutput {
        btc_latest_block_number: state.btc_db_utils.get_btc_latest_block_from_db()?.height,
        int_signed_transactions: if signed_txs.len() == 0 {
            vec![]
        } else {
            get_eth_signed_tx_info_from_eth_txs(
                &state.eth_signed_txs,
                &state
                    .btc_db_utils
                    .get_btc_canon_block_from_db()?
                    .get_btc_on_int_int_tx_infos(),
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                state.eth_db_utils.get_latest_eth_block_number()?,
            )?
        },
    };
    state.add_output_json_string(serde_json::to_string(&output)?)
}

pub fn get_btc_output_as_string<D: DatabaseInterface>(state: BtcState<D>) -> Result<String> {
    state.get_output_json_string()
}
