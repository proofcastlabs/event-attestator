use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::{
        any_sender::relay_transaction::RelayTransaction,
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
        eth_traits::EthTxInfoCompatible,
    },
    erc20_on_int::int::eth_tx_info::{EthOnIntEthTxInfo, EthOnIntEthTxInfos},
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct IntOutput {
    pub int_latest_block_number: usize,
    pub eth_signed_transactions: Vec<EthTxInfo>,
}

#[cfg(test)]
impl IntOutput {
    pub fn from_str(s: &str) -> Result<Self> {
        use serde_json::Value as JsonValue;
        #[derive(Deserialize)]
        struct TempStruct {
            int_latest_block_number: usize,
            eth_signed_transactions: Vec<JsonValue>,
        }
        let temp_struct = serde_json::from_str::<TempStruct>(s)?;
        let tx_infos = temp_struct
            .eth_signed_transactions
            .iter()
            .map(|json_value| EthTxInfo::from_str(&json_value.to_string()))
            .collect::<Result<Vec<EthTxInfo>>>()?;
        Ok(Self {
            eth_signed_transactions: tx_infos,
            int_latest_block_number: temp_struct.int_latest_block_number,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthTxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub eth_tx_hash: String,
    pub eth_tx_amount: String,
    pub eth_tx_recipient: String,
    pub witnessed_timestamp: u64,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub native_token_address: String,
    pub eth_signed_tx: Option<String>,
    pub any_sender_nonce: Option<u64>,
    pub eth_account_nonce: Option<u64>,
    pub eth_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
    pub any_sender_tx: Option<RelayTransaction>,
}

impl EthTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        evm_tx_info: &EthOnIntEthTxInfo,
        maybe_nonce: Option<u64>,
        eth_latest_block_number: usize,
    ) -> Result<EthTxInfo> {
        let nonce = maybe_nonce.ok_or(NoneError("No nonce for EVM output!"))?;
        Ok(EthTxInfo {
            eth_latest_block_number,
            broadcast: false,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            eth_signed_tx: tx.eth_tx_hex(),
            any_sender_tx: tx.any_sender_tx(),
            _id: if tx.is_any_sender() {
                format!("perc20-on-int-eth-any-sender-{}", nonce)
            } else {
                format!("perc20-on-int-eth-{}", nonce)
            },
            eth_tx_hash: format!("0x{}", tx.get_tx_hash()),
            eth_tx_amount: evm_tx_info.native_token_amount.to_string(),
            any_sender_nonce: if tx.is_any_sender() { maybe_nonce } else { None },
            eth_account_nonce: if tx.is_any_sender() { None } else { maybe_nonce },
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            host_token_address: format!("0x{}", hex::encode(&evm_tx_info.evm_token_address)),
            native_token_address: format!("0x{}", hex::encode(&evm_tx_info.eth_token_address)),
            originating_address: format!("0x{}", hex::encode(evm_tx_info.token_sender.as_bytes())),
            eth_tx_recipient: format!("0x{}", hex::encode(evm_tx_info.destination_address.as_bytes())),
            originating_tx_hash: format!("0x{}", hex::encode(evm_tx_info.originating_tx_hash.as_bytes())),
        })
    }
}

#[cfg(test)]
impl EthTxInfo {
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

pub fn get_eth_signed_tx_info_from_evm_txs(
    txs: &[EthTransaction],
    evm_tx_info: &EthOnIntEthTxInfos,
    eth_account_nonce: u64,
    use_any_sender_tx_type: bool,
    any_sender_nonce: u64,
    eth_latest_block_number: usize,
) -> Result<Vec<EthTxInfo>> {
    let number_of_txs = txs.len() as u64;
    let start_nonce = if use_any_sender_tx_type {
        info!("✔ Getting AnySender tx info from ETH txs...");
        if number_of_txs > any_sender_nonce {
            return Err("AnySencer account nonce has not been incremented correctly!".into());
        } else {
            any_sender_nonce - number_of_txs
        }
    } else {
        info!("✔ Getting EVM tx info from ETH txs...");
        if number_of_txs > eth_account_nonce {
            return Err("ETH account nonce has not been incremented correctly!".into());
        } else {
            eth_account_nonce - number_of_txs
        }
    };
    txs.iter()
        .enumerate()
        .map(|(i, tx)| {
            EthTxInfo::new(
                tx,
                &evm_tx_info[i],
                Some(start_nonce + i as u64),
                eth_latest_block_number,
            )
        })
        .collect::<Result<Vec<EthTxInfo>>>()
}

pub fn get_evm_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting EVM output json...");
    let output = serde_json::to_string(&IntOutput {
        int_latest_block_number: state.evm_db_utils.get_latest_eth_block_number()?,
        eth_signed_transactions: if state.erc20_on_int_eth_signed_txs.is_empty() {
            vec![]
        } else {
            get_eth_signed_tx_info_from_evm_txs(
                &state.erc20_on_int_eth_signed_txs,
                &state.erc20_on_int_eth_tx_infos,
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                false, // TODO Get this from state submission material when/if we support AnySender
                state.eth_db_utils.get_any_sender_nonce_from_db()?,
                state.eth_db_utils.get_latest_eth_block_number()?,
            )?
        },
    })?;
    info!("✔ EVM output: {}", output);
    Ok(output)
}
