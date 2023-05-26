use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::blockdata::transaction::Transaction as BtcTransaction;
use common::{
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_btc::{get_hex_tx_from_signed_btc_tx, BtcDbUtils, BtcTransactions};
use common_eth::{EthDbUtilsExt, EthState};
use serde::{Deserialize, Serialize};

use crate::eth::btc_tx_info::{BtcOnEthBtcTxInfo, BtcOnEthBtcTxInfos};

#[derive(Debug, Serialize, Deserialize)]
pub struct BtcTxInfo {
    pub btc_tx_hex: String,
    pub btc_tx_hash: String,
    pub btc_tx_amount: u64,
    pub btc_account_nonce: u64,
    pub btc_tx_recipient: String,
    pub signature_timestamp: u64,
    pub originating_tx_hash: String,
    pub originating_address: String,
}

impl BtcTxInfo {
    pub fn new(btc_tx: &BtcTransaction, btc_tx_info: &BtcOnEthBtcTxInfo, btc_account_nonce: u64) -> Result<BtcTxInfo> {
        Ok(BtcTxInfo {
            btc_account_nonce,
            btc_tx_hash: btc_tx.txid().to_string(),
            btc_tx_amount: btc_tx_info.amount_in_satoshis,
            btc_tx_hex: get_hex_tx_from_signed_btc_tx(btc_tx),
            btc_tx_recipient: btc_tx_info.recipient.clone(),
            originating_address: format!("0x{}", hex::encode(btc_tx_info.from.as_bytes())),
            originating_tx_hash: format!("0x{}", hex::encode(btc_tx_info.originating_tx_hash.as_bytes())),
            signature_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthOutput {
    pub eth_latest_block_number: usize,
    pub btc_signed_transactions: Vec<BtcTxInfo>,
}

pub fn get_btc_signed_tx_info_from_btc_txs(
    btc_account_nonce: u64,
    btc_txs: BtcTransactions,
    btc_tx_infos: &BtcOnEthBtcTxInfos,
) -> Result<Vec<BtcTxInfo>> {
    info!("✔ Getting BTC tx info from {} BTC tx(s)...", btc_txs.len());
    let num_btc_txs = btc_txs.len();
    let num_btc_tx_infos = btc_tx_infos.len();
    if num_btc_txs > num_btc_tx_infos {
        // NOTE: There CAN be fewer such as in the case of txs being filtered out for amounts being too low.
        return Err(format!(
            "There are MORE transactions than BTC tx infos! Num BTC txs: {}, Num BtcTxInfos: {}",
            num_btc_txs, num_btc_tx_infos
        )
        .into());
    };
    let start_nonce = btc_account_nonce - btc_txs.len() as u64;
    btc_txs
        .iter()
        .enumerate()
        .map(|(i, btc_tx)| BtcTxInfo::new(btc_tx, &btc_tx_infos.0[i], start_nonce + i as u64))
        .collect::<Result<Vec<_>>>()
}

pub fn get_eth_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting ETH output json...");
    let output = serde_json::to_string(&EthOutput {
        eth_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_usize(),
        btc_signed_transactions: if state.signed_txs.is_empty() {
            vec![]
        } else {
            get_btc_signed_tx_info_from_btc_txs(
                BtcDbUtils::new(state.db).get_btc_account_nonce_from_db()?,
                BtcTransactions::from_bytes(&state.signed_txs)?,
                &BtcOnEthBtcTxInfos::from_bytes(&state.tx_infos)?,
            )?
        },
    })?;
    info!("✔ ETH Output: {}", output);
    Ok(output)
}
