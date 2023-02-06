use std::time::{SystemTime, UNIX_EPOCH};

use common::{
    chains::eth::{
        any_sender::relay_transaction::RelayTransaction,
        eth_chain_id::EthChainId,
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::EthDbUtilsExt,
        eth_traits::EthTxInfoCompatible,
    },
    metadata::metadata_traits::ToMetadataChainId,
    state::EthState,
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

use crate::int::eth_tx_info::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos};

make_output_structs!(Int, Eth);

make_struct_with_test_assertions_on_equality_check!(
    struct EthTxInfo {
        _id: String,
        broadcast: bool,
        eth_tx_hash: String,
        eth_tx_amount: String,
        eth_tx_recipient: String,
        witnessed_timestamp: u64,
        host_token_address: String,
        originating_tx_hash: String,
        originating_address: String,
        native_token_address: String,
        destination_chain_id: String,
        eth_signed_tx: Option<String>,
        any_sender_nonce: Option<u64>,
        eth_account_nonce: Option<u64>,
        eth_latest_block_number: usize,
        broadcast_tx_hash: Option<String>,
        broadcast_timestamp: Option<String>,
        any_sender_tx: Option<RelayTransaction>,
    }
);

impl EthTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        tx_info: &Erc20OnIntEthTxInfo,
        maybe_nonce: Option<u64>,
        eth_latest_block_number: usize,
        eth_chain_id: &EthChainId,
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
            eth_tx_recipient: tx_info.destination_address.clone(),
            eth_tx_amount: tx_info.native_token_amount.to_string(),
            any_sender_nonce: if tx.is_any_sender() { maybe_nonce } else { None },
            eth_account_nonce: if tx.is_any_sender() { None } else { maybe_nonce },
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            host_token_address: format!("0x{}", hex::encode(tx_info.evm_token_address)),
            native_token_address: format!("0x{}", hex::encode(tx_info.eth_token_address)),
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender.as_bytes())),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash.as_bytes())),
            destination_chain_id: format!("0x{}", hex::encode(eth_chain_id.to_metadata_chain_id().to_bytes()?)),
        })
    }
}

pub fn get_eth_signed_tx_info_from_evm_txs(
    txs: &[EthTransaction],
    evm_tx_info: &Erc20OnIntEthTxInfos,
    eth_account_nonce: u64,
    use_any_sender_tx_type: bool,
    any_sender_nonce: u64,
    eth_latest_block_number: usize,
    eth_chain_id: &EthChainId,
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
                eth_chain_id,
            )
        })
        .collect::<Result<Vec<EthTxInfo>>>()
}

pub fn get_evm_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<IntOutput> {
    info!("✔ Getting INT output json...");
    let output = IntOutput {
        int_latest_block_number: state.evm_db_utils.get_latest_eth_block_number()?,
        eth_signed_transactions: if state.erc20_on_int_eth_signed_txs.is_empty() {
            vec![]
        } else {
            get_eth_signed_tx_info_from_evm_txs(
                &state.erc20_on_int_eth_signed_txs,
                &Erc20OnIntEthTxInfos::from_bytes(&state.tx_infos)?,
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                false, // TODO Get this from state submission material when/if we support AnySender
                state.eth_db_utils.get_any_sender_nonce_from_db()?,
                state.eth_db_utils.get_latest_eth_block_number()?,
                &state.eth_db_utils.get_eth_chain_id_from_db()?,
            )?
        },
    };
    info!("✔ EVM output: {}", output);
    Ok(output)
}
