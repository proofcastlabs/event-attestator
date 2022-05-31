#[cfg(test)]
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use derive_more::Constructor;
use rust_algorand::AlgorandSignedTransaction;
use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    int_on_algo::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Debug, Serialize, Deserialize, Constructor)]
pub struct IntOutput {
    pub int_latest_block_number: usize,
    pub algo_signed_transactions: Vec<IntTxInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntTxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub algo_tx_hash: String,
    pub algo_signed_tx: String,
    pub algo_tx_amount: String,
    pub algo_account_nonce: u64,
    pub witnessed_timestamp: u64,
    pub algo_tx_recipient: String,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub native_token_address: String,
    pub destination_chain_id: String,
    pub algo_latest_block_number: u64,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

impl IntTxInfo {
    pub fn new(
        signed_tx: &AlgorandSignedTransaction,
        tx_info: &IntOnAlgoAlgoTxInfo,
        nonce: u64,
        algo_latest_block_number: u64,
    ) -> Result<IntTxInfo> {
        Ok(IntTxInfo {
            broadcast: false,
            broadcast_tx_hash: None,
            algo_latest_block_number,
            broadcast_timestamp: None,
            algo_account_nonce: nonce,
            algo_signed_tx: signed_tx.to_hex()?,
            _id: format!("pint-on-algo-algo-{}", nonce),
            algo_tx_hash: signed_tx.transaction.to_id()?,
            algo_tx_amount: tx_info.host_token_amount.to_string(),
            host_token_address: format!("{}", tx_info.algo_asset_id),
            algo_tx_recipient: tx_info.destination_address.to_string(),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            native_token_address: format!("0x{}", hex::encode(&tx_info.int_token_address)),
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender.as_bytes())),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash.as_bytes())),
            destination_chain_id: format!("0x{}", hex::encode(&tx_info.destination_chain_id.to_bytes()?)),
        })
    }
}

pub fn get_int_signed_tx_info_from_int_txs(
    txs: &[AlgorandSignedTransaction],
    tx_infos: &IntOnAlgoAlgoTxInfos,
    algo_account_nonce: u64,
    algo_latest_block_num: u64,
) -> Result<Vec<IntTxInfo>> {
    let number_of_txs = txs.len() as u64;
    let start_nonce = algo_account_nonce - number_of_txs;
    info!("✔ Getting INT tx info from ALGO txs...");
    if number_of_txs > algo_account_nonce {
        return Err("ALGO account nonce has not been incremented correctly!".into());
    };
    if number_of_txs != tx_infos.len() as u64 {
        return Err("Number of txs does not match number of tx infos!".into());
    };
    txs.iter()
        .zip(tx_infos.iter())
        .enumerate()
        .map(|(i, (tx, info))| IntTxInfo::new(tx, info, start_nonce + i as u64, algo_latest_block_num))
        .collect::<Result<Vec<_>>>()
}

pub fn get_int_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting INT output json...");
    let txs = state.algo_signed_txs.clone();
    let int_latest_block_num = state.eth_db_utils.get_latest_eth_block_number()?;
    let output = if !txs.is_empty() {
        IntOutput::new(
            int_latest_block_num,
            get_int_signed_tx_info_from_int_txs(
                &txs,
                &state.int_on_algo_algo_tx_infos,
                state.algo_db_utils.get_algo_account_nonce()?,
                state.algo_db_utils.get_latest_block_number()?,
            )?,
        )
    } else {
        IntOutput::new(int_latest_block_num, vec![])
    };
    Ok(serde_json::to_string(&output)?)
}

#[cfg(test)]
impl FromStr for IntOutput {
    type Err = crate::errors::AppError;

    fn from_str(s: &str) -> Result<Self> {
        use serde_json::Value as JsonValue;
        #[derive(Deserialize)]
        struct TempStruct {
            int_latest_block_number: usize,
            algo_signed_transactions: Vec<JsonValue>,
        }
        let temp_struct = serde_json::from_str::<TempStruct>(s)?;
        let tx_infos = temp_struct
            .algo_signed_transactions
            .iter()
            .map(|json_value| IntTxInfo::from_str(&json_value.to_string()))
            .collect::<Result<Vec<IntTxInfo>>>()?;
        Ok(Self {
            algo_signed_transactions: tx_infos,
            int_latest_block_number: temp_struct.int_latest_block_number,
        })
    }
}

#[cfg(test)]
impl FromStr for IntTxInfo {
    type Err = crate::errors::AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}
