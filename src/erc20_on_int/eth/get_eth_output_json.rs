use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    chains::eth::{
        any_sender::relay_transaction::RelayTransaction,
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
        eth_traits::EthTxInfoCompatible,
        eth_utils::convert_eth_address_to_string,
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_int::eth::int_tx_info::{
        Erc20OnIntIntTxInfo as EthOnIntEvmTxInfo,
        Erc20OnIntIntTxInfos as EthOnIntEvmTxInfos,
    },
    traits::DatabaseInterface,
    types::{NoneError, Result},
};

make_output_structs!(Eth, Int);

make_struct_with_test_assertions_on_equality_check!(
    struct IntTxInfo {
        _id: String,
        broadcast: bool,
        int_tx_hash: String,
        int_tx_amount: String,
        int_tx_recipient: String,
        witnessed_timestamp: u64,
        host_token_address: String,
        originating_tx_hash: String,
        originating_address: String,
        native_token_address: String,
        destination_chain_id: String,
        int_signed_tx: Option<String>,
        any_sender_nonce: Option<u64>,
        int_account_nonce: Option<u64>,
        int_latest_block_number: usize,
        broadcast_tx_hash: Option<String>,
        broadcast_timestamp: Option<String>,
        any_sender_tx: Option<RelayTransaction>,
    }
);

impl IntTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        tx_info: &EthOnIntEvmTxInfo,
        maybe_nonce: Option<u64>,
        int_latest_block_number: usize,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<IntTxInfo> {
        let nonce = maybe_nonce.ok_or_else(|| NoneError("No nonce for EVM output!"))?;
        Ok(IntTxInfo {
            int_latest_block_number,
            broadcast: false,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            int_signed_tx: tx.eth_tx_hex(),
            any_sender_tx: tx.any_sender_tx(),
            _id: if tx.is_any_sender() {
                format!("perc20-on-int-int-any-sender-{}", nonce)
            } else {
                format!("perc20-on-int-int-{}", nonce)
            },
            int_tx_hash: format!("0x{}", tx.get_tx_hash()),
            int_tx_recipient: tx_info.destination_address.clone(),
            any_sender_nonce: if tx.is_any_sender() { maybe_nonce } else { None },
            int_account_nonce: if tx.is_any_sender() { None } else { maybe_nonce },
            host_token_address: convert_eth_address_to_string(&tx_info.evm_token_address),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            native_token_address: format!("0x{}", hex::encode(tx_info.eth_token_address)),
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender.as_bytes())),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash.as_bytes())),
            destination_chain_id: format!("0x{}", hex::encode(&tx_info.destination_chain_id.to_bytes()?)),
            int_tx_amount: dictionary
                .convert_eth_amount_to_evm_amount(&tx_info.eth_token_address, tx_info.native_token_amount)?
                .to_string(),
        })
    }
}

pub fn get_evm_signed_tx_info_from_evm_txs(
    txs: &[EthTransaction],
    evm_tx_info: &EthOnIntEvmTxInfos,
    eth_account_nonce: u64,
    use_any_sender_tx_type: bool,
    any_sender_nonce: u64,
    eth_latest_block_number: usize,
    dictionary: &EthEvmTokenDictionary,
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
        info!("✔ Getting EVM tx info from ETH txs...");
        if number_of_txs > eth_account_nonce {
            return Err("INT account nonce has not been incremented correctly!".into());
        } else {
            eth_account_nonce - number_of_txs
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
                dictionary,
            )
        })
        .collect::<Result<Vec<IntTxInfo>>>()
}

pub fn get_eth_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting ETH output json...");
    let output = serde_json::to_string(&EthOutput {
        eth_latest_block_number: state.eth_db_utils.get_latest_eth_block_number()?,
        int_signed_transactions: if state.erc20_on_int_int_signed_txs.is_empty() {
            vec![]
        } else {
            get_evm_signed_tx_info_from_evm_txs(
                &state.erc20_on_int_int_signed_txs,
                &state.erc20_on_int_int_tx_infos,
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
