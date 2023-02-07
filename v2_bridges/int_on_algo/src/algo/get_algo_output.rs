use std::{
    fmt::{Display, Formatter},
    time::{SystemTime, UNIX_EPOCH},
};

use algorand::AlgoState;
use common::{
    chains::eth::{
        eth_crypto::eth_transaction::EthTransaction as IntTransaction,
        eth_database_utils::EthDbUtilsExt,
        eth_traits::EthTxInfoCompatible,
    },
    traits::DatabaseInterface,
    types::Result,
};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::algo::int_tx_info::{IntOnAlgoIntTxInfo, IntOnAlgoIntTxInfos};

#[derive(Debug, Clone, Eq, Default, PartialEq, Serialize, Deserialize, Constructor)]
pub struct AlgoOutput {
    pub algo_latest_block_number: u64,
    pub int_signed_transactions: Vec<IntTxOutput>,
}

impl Display for AlgoOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(json_string) => write!(f, "{}", json_string),
            Err(_) => write!(f, "Error getting algo output!"),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntTxOutput {
    pub _id: String,
    pub broadcast: bool,
    pub int_tx_hash: String,
    pub int_tx_amount: String,
    pub int_account_nonce: u64,
    pub int_tx_recipient: String,
    pub witnessed_timestamp: u64,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub destination_chain_id: String,
    pub native_token_address: String,
    pub int_signed_tx: Option<String>,
    pub int_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

#[cfg(test)]
impl AlgoOutput {
    pub fn from_str(s: &str) -> Result<Self> {
        use serde_json::Value as JsonValue;
        #[derive(Deserialize)]
        struct TempStruct {
            algo_latest_block_number: u64,
            int_signed_transactions: Vec<JsonValue>,
        }
        let temp_struct = serde_json::from_str::<TempStruct>(s)?;
        let tx_infos = temp_struct
            .int_signed_transactions
            .iter()
            .map(|json_value| IntTxOutput::from_str(&json_value.to_string()))
            .collect::<Result<Vec<IntTxOutput>>>()?;
        Ok(Self {
            int_signed_transactions: tx_infos,
            algo_latest_block_number: temp_struct.algo_latest_block_number,
        })
    }
}

#[cfg(test)]
impl IntTxOutput {
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

impl IntTxOutput {
    pub fn new(
        tx: &IntTransaction,
        tx_info: &IntOnAlgoIntTxInfo,
        nonce: u64,
        int_latest_block_number: usize,
    ) -> Result<IntTxOutput> {
        Ok(IntTxOutput {
            broadcast: false,
            int_latest_block_number,
            broadcast_tx_hash: None,
            int_account_nonce: nonce,
            broadcast_timestamp: None,
            int_signed_tx: tx.eth_tx_hex(),
            _id: format!("pint-on-algo-int-{}", nonce),
            int_tx_hash: format!("0x{}", tx.get_tx_hash()),
            originating_address: tx_info.token_sender.to_string(),
            int_tx_amount: tx_info.native_token_amount.to_string(),
            host_token_address: format!("{}", tx_info.algo_asset_id),
            int_tx_recipient: tx_info.destination_address.to_string(),
            originating_tx_hash: tx_info.originating_tx_hash.to_string(),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            native_token_address: format!("0x{}", hex::encode(tx_info.int_token_address)),
            destination_chain_id: format!("0x{}", hex::encode(tx_info.destination_chain_id.to_bytes()?)),
        })
    }
}

pub fn get_int_signed_tx_info_from_algo_txs(
    signed_txs: &[IntTransaction],
    tx_infos: &IntOnAlgoIntTxInfos,
    int_account_nonce: u64,
    int_latest_block_number: usize,
) -> Result<Vec<IntTxOutput>> {
    info!("✔ Getting INT signed tx info from ALGO txs...");
    let number_of_txs = signed_txs.len() as u64;
    let start_nonce = if number_of_txs > int_account_nonce {
        return Err("INT account nonce has not been incremented correctly!".into());
    } else {
        info!("✔ Getting INT tx info from ALGO txs...");
        int_account_nonce - number_of_txs
    };
    if number_of_txs != tx_infos.len() as u64 {
        return Err("Number of signed transactions does not match number of tx infos!".into());
    };
    signed_txs
        .iter()
        .zip(tx_infos.iter())
        .enumerate()
        .map(|(i, (signed_tx, tx_info))| {
            IntTxOutput::new(signed_tx, tx_info, start_nonce + i as u64, int_latest_block_number)
        })
        .collect::<Result<Vec<_>>>()
}

pub fn get_algo_output<D: DatabaseInterface>(state: AlgoState<D>) -> Result<String> {
    info!("✔ Getting ALGO output...");
    let signed_txs = state.eth_signed_txs.clone();
    let tx_infos = IntOnAlgoIntTxInfos::from_bytes(&state.tx_infos)?;
    let output = AlgoOutput {
        algo_latest_block_number: state.algo_db_utils.get_latest_block_number()?,
        int_signed_transactions: if signed_txs.is_empty() {
            vec![]
        } else {
            get_int_signed_tx_info_from_algo_txs(
                &signed_txs,
                &tx_infos,
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                state.eth_db_utils.get_latest_eth_block_number()?,
            )?
        },
    };
    Ok(output.to_string())
}
