use std::{
    fmt::{Display, Formatter},
    time::{SystemTime, UNIX_EPOCH},
};

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::{
    chains::{algo::algo_state::AlgoState, eth::any_sender::relay_transaction::RelayTransaction},
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct AlgoOutput {
    pub algo_latest_block_number: u64,
    pub int_signed_transactions: Vec<IntTxInfo>,
}

impl Display for AlgoOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(json_string) => write!(f, "{}", json_string),
            Err(_) => write!(f, "Error getting algo output!"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntTxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub int_tx_hash: String,
    pub int_tx_amount: String,
    pub int_tx_recipient: String,
    pub witnessed_timestamp: u64,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub native_token_address: String,
    pub int_signed_tx: Option<String>,
    pub any_sender_nonce: Option<u64>,
    pub int_account_nonce: Option<u64>,
    pub int_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
    pub any_sender_tx: Option<RelayTransaction>,
}

/*
#[cfg(test)]
impl AlgoOutput {
    pub fn from_str(s: &str) -> Result<Self> {
        use serde_json::Value as JsonValue;
        #[derive(Deserialize)]
        struct TempStruct {
            evm_latest_block_number: usize,
            int_signed_transactions: Vec<JsonValue>,
        }
        let temp_struct = serde_json::from_str::<TempStruct>(s)?;
        let tx_infos = temp_struct
            .int_signed_transactions
            .iter()
            .map(|json_value| IntTxInfo::from_str(&json_value.to_string()))
            .collect::<Result<Vec<IntTxInfo>>>()?;
        Ok(Self {
            int_signed_transactions: tx_infos,
            evm_latest_block_number: temp_struct.evm_latest_block_number,
        })
    }
}


#[cfg(test)]
impl IntTxInfo {
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

impl IntTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        evm_tx_info: &IntOnEvmIntTxInfo,
        maybe_nonce: Option<u64>,
        int_latest_block_number: usize,
    ) -> Result<IntTxInfo> {
        let nonce = maybe_nonce.ok_or(NoneError("No nonce for EVM output!"))?;
        Ok(IntTxInfo {
            broadcast: false,
            int_latest_block_number,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            int_signed_tx: tx.eth_tx_hex(),
            any_sender_tx: tx.any_sender_tx(),
            _id: if tx.is_any_sender() {
                format!("pint-on-evm-int-any-sender-{}", nonce)
            } else {
                format!("pint-on-evm-int-{}", nonce)
            },
            int_tx_hash: format!("0x{}", tx.get_tx_hash()),
            int_tx_amount: evm_tx_info.native_token_amount.to_string(),
            any_sender_nonce: if tx.is_any_sender() { maybe_nonce } else { None },
            int_account_nonce: if tx.is_any_sender() { None } else { maybe_nonce },
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            host_token_address: format!("0x{}", hex::encode(&evm_tx_info.evm_token_address)),
            native_token_address: format!("0x{}", hex::encode(&evm_tx_info.eth_token_address)),
            originating_address: format!("0x{}", hex::encode(evm_tx_info.token_sender.as_bytes())),
            int_tx_recipient: format!("0x{}", hex::encode(evm_tx_info.destination_address.as_bytes())),
            originating_tx_hash: format!("0x{}", hex::encode(evm_tx_info.originating_tx_hash.as_bytes())),
        })
    }
}

pub fn get_int_signed_tx_info_from_evm_txs(
    txs: &[EthTransaction],
    evm_tx_info: &IntOnEvmIntTxInfos,
    int_account_nonce: u64,
    use_any_sender_tx_type: bool,
    any_sender_nonce: u64,
    eth_latest_block_number: usize,
) -> Result<Vec<IntTxInfo>> {
    let number_of_txs = txs.len() as u64;
    let start_nonce = if use_any_sender_tx_type {
        info!("✔ Getting AnySender tx info from ETH txs...");
        if number_of_txs > any_sender_nonce {
            return Err("AnySender account nonce has not been incremented correctly!".into());
        } else {
            any_sender_nonce - number_of_txs
        }
    } else {
        info!("✔ Getting INT tx info from EVM txs...");
        if number_of_txs > int_account_nonce {
            return Err("INT account nonce has not been incremented correctly!".into());
        } else {
            int_account_nonce - number_of_txs
        }
    };
    txs.iter()
        .enumerate()
        .map(|(i, tx)| {
            IntTxInfo::new(
                tx,
                &evm_tx_info[i],
                Some(start_nonce + i as u64),
                eth_latest_block_number,
            )
        })
        .collect::<Result<Vec<IntTxInfo>>>()
}
*/

pub fn get_algo_output<D: DatabaseInterface>(state: AlgoState<D>) -> Result<String> {
    info!("✔ Getting ALGO output...");
    let json = AlgoOutput::new(
        state.algo_db_utils.get_latest_block_number()?,
        vec![], // FIXME
    );
    info!("✔ ALGO output: {json}");
    Ok(json.to_string())
    /*
    let output = serde_json::to_string(&AlgoOutput {
        evm_latest_block_number: state.evm_db_utils.get_latest_eth_block_number()?,
        int_signed_transactions: if state.int_on_evm_int_signed_txs.is_empty() {
            vec![]
        } else {
            get_int_signed_tx_info_from_evm_txs(
                &state.int_on_evm_int_signed_txs,
                &state.int_on_evm_int_tx_infos,
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                false, // TODO Get this from state submission material when/if we support AnySender
                state.eth_db_utils.get_any_sender_nonce_from_db()?,
                state.eth_db_utils.get_latest_eth_block_number()?,
            )?
        },
    })?;
    */
}
