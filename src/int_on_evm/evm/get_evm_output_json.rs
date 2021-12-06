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
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EvmOutput {
    pub evm_latest_block_number: usize,
    pub int_signed_transactions: Vec<IntTxInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub evm_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
    pub any_sender_tx: Option<RelayTransaction>,
}

impl IntTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        evm_tx_info: &IntOnEvmIntTxInfo,
        maybe_nonce: Option<u64>,
        evm_latest_block_number: usize,
    ) -> Result<IntTxInfo> {
        let nonce = maybe_nonce.ok_or(NoneError("No nonce for EVM output!"))?;
        Ok(IntTxInfo {
            evm_latest_block_number,
            broadcast: false,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            int_signed_tx: tx.eth_tx_hex(),
            any_sender_tx: tx.any_sender_tx(),
            _id: if tx.is_any_sender() {
                format!("perc20-on-evm-eth-any-sender-{}", nonce)
            } else {
                format!("perc20-on-evm-eth-{}", nonce)
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
    let number_of_txs = txs.len();
    let start_nonce = if use_any_sender_tx_type {
        info!("✔ Getting AnySender tx info from ETH txs...");
        if any_sender_nonce == 0 {
            0
        } else {
            any_sender_nonce - number_of_txs as u64
        }
    } else {
        info!("✔ Getting INT tx info from EVM txs...");
        if int_account_nonce == 0 {
            0
        } else {
            int_account_nonce - number_of_txs as u64
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

pub fn get_evm_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting EVM output json...");
    let output = serde_json::to_string(&EvmOutput {
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
    info!("✔ EVM output: {}", output);
    Ok(output)
}
