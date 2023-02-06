use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::blockdata::transaction::Transaction as BtcTransaction;
use common::{
    chains::btc::btc_utils::get_hex_tx_from_signed_btc_tx,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};
use serde::{Deserialize, Serialize};

use crate::eos::btc_tx_info::{BtcOnEosBtcTxInfo, BtcOnEosBtcTxInfos};

#[derive(Debug, Serialize, Deserialize)]
pub struct EosOutput {
    pub btc_signed_transactions: Vec<BtcTxInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BtcTxInfo {
    pub btc_tx_hex: String,
    pub btc_tx_amount: u64,
    pub btc_tx_hash: String,
    pub signature_timestamp: u64,
    pub btc_account_nonce: u64,
    pub btc_tx_recipient: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
}

impl BtcTxInfo {
    pub fn new(btc_tx: &BtcTransaction, btc_tx_info: &BtcOnEosBtcTxInfo, btc_account_nonce: u64) -> Result<BtcTxInfo> {
        Ok(BtcTxInfo {
            btc_account_nonce,
            btc_tx_amount: btc_tx_info.amount,
            btc_tx_hash: btc_tx.txid().to_string(),
            btc_tx_recipient: btc_tx_info.recipient.clone(),
            btc_tx_hex: get_hex_tx_from_signed_btc_tx(btc_tx),
            originating_address: format!("{}", btc_tx_info.from),
            originating_tx_hash: format!("{}", btc_tx_info.originating_tx_id),
            signature_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        })
    }
}

pub fn get_btc_signed_tx_info_from_btc_txs(
    btc_account_nonce: u64,
    btc_txs: &[BtcTransaction],
    btc_tx_infos: &BtcOnEosBtcTxInfos,
) -> Result<Vec<BtcTxInfo>> {
    info!("✔ Getting BTC tx info from BTC txs...");
    let num_btc_txs = btc_txs.len();
    let num_btc_tx_infos = btc_tx_infos.len();
    if num_btc_txs != num_btc_tx_infos {
        return Err(format!(
            "BTC tx mismatch. BTC txs: #{}, RedeemInfos: #{}",
            num_btc_txs, num_btc_tx_infos
        )
        .into());
    };
    let start_nonce = btc_account_nonce - btc_txs.len() as u64;
    btc_txs
        .iter()
        .enumerate()
        .map(|(i, btc_tx)| BtcTxInfo::new(btc_tx, &btc_tx_infos.0[i], start_nonce + i as u64))
        .collect()
}

pub fn get_eos_output<D: DatabaseInterface>(state: EosState<D>) -> Result<String> {
    info!("✔ Getting EOS output json...");
    let output = serde_json::to_string(&EosOutput {
        btc_signed_transactions: match &state.btc_on_eos_signed_txs.len() {
            0 => vec![],
            _ => get_btc_signed_tx_info_from_btc_txs(
                state.btc_db_utils.get_btc_account_nonce_from_db()?,
                &state.btc_on_eos_signed_txs,
                &BtcOnEosBtcTxInfos::from_bytes(&state.tx_infos)?,
            )?,
        },
    })?;
    info!("✔ EOS output: {}", output);
    Ok(output)
}
