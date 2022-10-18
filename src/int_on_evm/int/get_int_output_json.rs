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
    dictionaries::eth_evm::EthEvmTokenDictionary,
    int_on_evm::int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos},
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct IntOutput {
    pub int_latest_block_number: usize,
    pub evm_signed_transactions: Vec<EvmTxInfo>,
}

#[cfg(test)]
impl IntOutput {
    pub fn from_str(s: &str) -> Result<Self> {
        use serde_json::Value as JsonValue;
        #[derive(Deserialize)]
        struct TempStruct {
            int_latest_block_number: usize,
            evm_signed_transactions: Vec<JsonValue>,
        }
        let temp_struct = serde_json::from_str::<TempStruct>(s)?;
        let tx_infos = temp_struct
            .evm_signed_transactions
            .iter()
            .map(|json_value| EvmTxInfo::from_str(&json_value.to_string()))
            .collect::<Result<Vec<EvmTxInfo>>>()?;
        Ok(Self {
            evm_signed_transactions: tx_infos,
            int_latest_block_number: temp_struct.int_latest_block_number,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvmTxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub evm_tx_hash: String,
    pub evm_tx_amount: String,
    pub evm_tx_recipient: String,
    pub witnessed_timestamp: u64,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub destination_chain_id: String,
    pub native_token_address: String,
    pub evm_signed_tx: Option<String>,
    pub any_sender_nonce: Option<u64>,
    pub evm_account_nonce: Option<u64>,
    pub evm_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
    pub any_sender_tx: Option<RelayTransaction>,
}

#[cfg(test)]
impl EvmTxInfo {
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

impl EvmTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        tx_info: &IntOnEvmEvmTxInfo,
        maybe_nonce: Option<u64>,
        evm_latest_block_number: usize,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmTxInfo> {
        let nonce = maybe_nonce.ok_or_else(|| NoneError("No nonce for EVM output!"))?;
        Ok(EvmTxInfo {
            evm_latest_block_number,
            broadcast: false,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            evm_signed_tx: tx.eth_tx_hex(),
            any_sender_tx: tx.any_sender_tx(),
            _id: if tx.is_any_sender() {
                format!("pint-on-evm-evm-any-sender-{}", nonce)
            } else {
                format!("pint-on-evm-evm-{}", nonce)
            },
            evm_tx_hash: format!("0x{}", tx.get_tx_hash()),
            any_sender_nonce: if tx.is_any_sender() { maybe_nonce } else { None },
            evm_account_nonce: if tx.is_any_sender() { None } else { maybe_nonce },
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            host_token_address: format!("0x{}", hex::encode(tx_info.evm_token_address)),
            native_token_address: format!("0x{}", hex::encode(tx_info.eth_token_address)),
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender.as_bytes())),
            evm_tx_recipient: format!("0x{}", hex::encode(tx_info.destination_address.as_bytes())),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash.as_bytes())),
            destination_chain_id: format!("0x{}", hex::encode(&tx_info.destination_chain_id.to_bytes()?)),
            evm_tx_amount: dictionary
                .convert_eth_amount_to_evm_amount(&tx_info.eth_token_address, tx_info.native_token_amount)?
                .to_string(),
        })
    }
}

pub fn get_evm_signed_tx_info_from_int_txs(
    txs: &[EthTransaction],
    tx_info: &IntOnEvmEvmTxInfos,
    eth_account_nonce: u64,
    use_any_sender_tx_type: bool,
    any_sender_nonce: u64,
    eth_latest_block_number: usize,
    dictionary: &EthEvmTokenDictionary,
) -> Result<Vec<EvmTxInfo>> {
    let number_of_txs = txs.len() as u64;
    let start_nonce = if use_any_sender_tx_type {
        info!("✔ Getting AnySender tx info from ETH txs...");
        if number_of_txs > any_sender_nonce {
            return Err("AnySender account nonce has not been incremented correctly!".into());
        } else {
            any_sender_nonce - number_of_txs
        }
    } else {
        info!("✔ Getting EVM tx info from ETH txs...");
        if number_of_txs > eth_account_nonce {
            return Err("Eth account nonce has not been incremented correctly!".into());
        } else {
            eth_account_nonce - number_of_txs
        }
    };
    txs.iter()
        .enumerate()
        .map(|(i, tx)| {
            EvmTxInfo::new(
                tx,
                &tx_info[i],
                Some(start_nonce + i as u64),
                eth_latest_block_number,
                dictionary,
            )
        })
        .collect::<Result<Vec<EvmTxInfo>>>()
}

pub fn get_int_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting ETH output json...");
    let output = serde_json::to_string(&IntOutput {
        int_latest_block_number: state.eth_db_utils.get_latest_eth_block_number()?,
        evm_signed_transactions: if state.int_on_evm_evm_signed_txs.is_empty() {
            info!("✔ No `int-on-evm-evm` signed transactions ∴ no txs to output!");
            vec![]
        } else {
            get_evm_signed_tx_info_from_int_txs(
                &state.int_on_evm_evm_signed_txs,
                &state.int_on_evm_evm_tx_infos,
                state.evm_db_utils.get_eth_account_nonce_from_db()?,
                false, // TODO Get this from state submission material when/if we support AnySender
                state.evm_db_utils.get_any_sender_nonce_from_db()?,
                state.evm_db_utils.get_latest_eth_block_number()?,
                &EthEvmTokenDictionary::get_from_db(state.db)?,
            )?
        },
    })?;
    info!("✔ ETH output: {}", output);
    Ok(output)
}
