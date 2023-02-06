use common::{
    chains::{
        btc::btc_constants::PLACEHOLDER_BTC_ADDRESS,
        eth::{
            any_sender::relay_transaction::RelayTransaction,
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::EthDbUtilsExt,
            eth_traits::EthTxInfoCompatible,
        },
    },
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
    utils::get_unix_timestamp,
};
use serde::{Deserialize, Serialize};

use crate::btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos};

#[derive(Debug, Serialize, Deserialize)]
pub struct EthTxInfo {
    pub eth_tx_hex: Option<String>,
    pub eth_tx_hash: String,
    pub eth_tx_amount: String,
    pub eth_account_nonce: Option<u64>,
    pub eth_tx_recipient: String,
    pub signature_timestamp: u64,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub any_sender_tx: Option<RelayTransaction>,
    pub any_sender_nonce: Option<u64>,
}

impl EthTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        eth_tx_info: &BtcOnEthEthTxInfo,
        nonce: Option<u64>,
    ) -> Result<EthTxInfo> {
        let default_address = PLACEHOLDER_BTC_ADDRESS.to_string();
        let retrieved_address = eth_tx_info.originating_tx_address.to_string();
        let address_string = match default_address == retrieved_address {
            false => retrieved_address,
            true => "✘ Could not retrieve sender address".to_string(),
        };

        Ok(EthTxInfo {
            eth_tx_hex: tx.eth_tx_hex(),
            any_sender_tx: tx.any_sender_tx(),
            originating_address: address_string,
            signature_timestamp: get_unix_timestamp()?,
            eth_tx_amount: eth_tx_info.amount.to_string(),
            eth_tx_hash: format!("0x{}", tx.get_tx_hash()),
            any_sender_nonce: if tx.is_any_sender() { nonce } else { None },
            originating_tx_hash: eth_tx_info.originating_tx_hash.to_string(),
            eth_account_nonce: if tx.is_any_sender() { None } else { nonce },
            eth_tx_recipient: format!("0x{}", hex::encode(eth_tx_info.destination_address.as_bytes())),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BtcOutput {
    pub btc_latest_block_number: u64,
    pub eth_signed_transactions: Vec<EthTxInfo>,
}

pub fn get_eth_signed_tx_info_from_eth_txs(
    eth_txs: &[EthTransaction],
    eth_tx_infos: &[BtcOnEthEthTxInfo],
    eth_account_nonce: u64,
    use_any_sender_tx_type: bool,
    any_sender_nonce: u64,
) -> Result<Vec<EthTxInfo>> {
    let start_nonce = if use_any_sender_tx_type {
        info!("✔ Getting AnySender tx info from ETH txs...");
        any_sender_nonce - eth_txs.len() as u64
    } else {
        info!("✔ Getting ETH tx info from ETH txs...");
        eth_account_nonce - eth_txs.len() as u64
    };

    eth_txs
        .iter()
        .enumerate()
        .map(|(i, tx)| EthTxInfo::new(tx, &eth_tx_infos[i], Some(start_nonce + i as u64)))
        .collect::<Result<Vec<EthTxInfo>>>()
}

pub fn create_btc_output_json_and_put_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Getting BTC output json and putting in state...");
    Ok(serde_json::to_string(&BtcOutput {
        btc_latest_block_number: state.btc_db_utils.get_btc_latest_block_from_db()?.height,
        eth_signed_transactions: match &state.eth_signed_txs.len() {
            0 => vec![],
            _ => get_eth_signed_tx_info_from_eth_txs(
                &state.eth_signed_txs,
                &BtcOnEthEthTxInfos::from_bytes(
                    &state.btc_db_utils.get_btc_canon_block_from_db()?.get_tx_info_bytes(),
                )?,
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                state.use_any_sender_tx_type(),
                state.eth_db_utils.get_any_sender_nonce_from_db()?,
            )?,
        },
    })?)
    .and_then(|output| state.add_output_json_string(output))
}

pub fn get_btc_output_as_string<D>(state: BtcState<D>) -> Result<String>
where
    D: DatabaseInterface,
{
    info!("✔ Getting BTC output as string...");
    let output = state.get_output_json_string()?;
    info!("✔ BTC Output: {}", output);
    Ok(output)
}
