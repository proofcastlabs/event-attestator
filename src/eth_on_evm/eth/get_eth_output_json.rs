use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    chains::{
        eth::{
            any_sender::relay_transaction::RelayTransaction,
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::{
                get_any_sender_nonce_from_db,
                get_eth_account_nonce_from_db,
                get_latest_eth_block_number,
            },
            eth_state::EthState,
            eth_traits::EthTxInfoCompatible,
        },
        evm::eth_database_utils::get_latest_eth_block_number as get_latest_evm_block_number,
    },
    eth_on_evm::eth::evm_tx_info::{EthOnEvmEvmTxInfo, EthOnEvmEvmTxInfos},
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EthOutput {
    pub eth_latest_block_number: usize,
    pub evm_signed_transactions: Vec<EvmTxInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvmTxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub evm_tx_hash: String,
    pub evm_tx_amount: String,
    pub eth_tx_amount: String,
    pub evm_tx_recipient: String,
    pub witnessed_timestamp: u64,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub native_token_address: String,
    pub evm_signed_tx: Option<String>,
    pub any_sender_nonce: Option<u64>,
    pub evm_account_nonce: Option<u64>,
    pub evm_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
    pub any_sender_tx: Option<RelayTransaction>,
}

impl EvmTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        evm_tx_info: &EthOnEvmEvmTxInfo,
        maybe_nonce: Option<u64>,
        evm_latest_block_number: usize,
    ) -> Result<EvmTxInfo> {
        let nonce = maybe_nonce.ok_or(NoneError("No nonce for EVM output!"))?;
        Ok(EvmTxInfo {
            evm_latest_block_number,
            broadcast: false,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            evm_signed_tx: tx.eth_tx_hex(),
            any_sender_tx: tx.any_sender_tx(),
            _id: if tx.is_any_sender() {
                format!("eth-on-evm-any-sender-{}", nonce)
            } else {
                format!("eth-on-evm-evm-{}", nonce)
            },
            evm_tx_hash: format!("0x{}", tx.get_tx_hash()),
            originating_address: evm_tx_info.token_sender.to_string(),
            evm_tx_amount: evm_tx_info.token_amount.to_string(),
            eth_tx_amount: evm_tx_info.token_amount.to_string(),
            host_token_address: format!("0x{}", hex::encode(&evm_tx_info.eth_token_address)),
            originating_tx_hash: evm_tx_info.originating_tx_hash.to_string(),
            any_sender_nonce: if tx.is_any_sender() { maybe_nonce } else { None },
            evm_account_nonce: if tx.is_any_sender() { None } else { maybe_nonce },
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            evm_tx_recipient: format!("0x{}", hex::encode(evm_tx_info.destination_address.as_bytes())),
            native_token_address: format!("0x{}", hex::encode(&evm_tx_info.evm_token_address)),
        })
    }
}

pub fn get_evm_signed_tx_info_from_evm_txs(
    txs: &[EthTransaction],
    evm_tx_info: &EthOnEvmEvmTxInfos,
    eth_account_nonce: u64,
    use_any_sender_tx_type: bool,
    any_sender_nonce: u64,
    eth_latest_block_number: usize,
) -> Result<Vec<EvmTxInfo>> {
    let start_nonce = if use_any_sender_tx_type {
        info!("✔ Getting AnySender tx info from ETH txs...");
        any_sender_nonce - txs.len() as u64
    } else {
        info!("✔ Getting EVM tx info from ETH txs...");
        eth_account_nonce - txs.len() as u64
    };
    txs.iter()
        .enumerate()
        .map(|(i, tx)| {
            EvmTxInfo::new(
                tx,
                &evm_tx_info[i],
                Some(start_nonce + i as u64),
                eth_latest_block_number,
            )
        })
        .collect::<Result<Vec<EvmTxInfo>>>()
}

pub fn get_eth_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting ETH output json...");
    let output = serde_json::to_string(&EthOutput {
        eth_latest_block_number: get_latest_eth_block_number(&state.db)?,
        evm_signed_transactions: if state.eth_on_evm_evm_signed_txs.is_empty() {
            vec![]
        } else {
            get_evm_signed_tx_info_from_evm_txs(
                &state.eth_on_evm_evm_signed_txs,
                &state.eth_on_evm_evm_tx_infos,
                get_eth_account_nonce_from_db(&state.db)?,
                false, // TODO Get this from state submission material when/if we support AnySender
                get_any_sender_nonce_from_db(&state.db)?,
                get_latest_evm_block_number(&state.db)?,
            )?
        },
    })?;
    info!("✔ ETH output: {}", output);
    Ok(output)
}
