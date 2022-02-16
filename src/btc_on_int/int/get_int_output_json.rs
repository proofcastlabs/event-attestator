use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::blockdata::transaction::Transaction as BtcTransaction;
use serde::{Deserialize, Serialize};

use crate::{
    btc_on_int::int::btc_tx_info::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
    chains::{
        btc::btc_utils::get_hex_tx_from_signed_btc_tx,
        eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    },
    traits::DatabaseInterface,
    types::Result,
};

// FIXME Standardize this with existing output formats!
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct IntOutput {
    pub btc_tx_hex: String,
    pub btc_tx_hash: String,
    pub btc_tx_amount: u64,
    pub btc_account_nonce: u64,
    pub btc_tx_recipient: String,
    pub signature_timestamp: u64,
    pub originating_tx_hash: String,
    pub originating_address: String,
}

impl IntOutput {
    pub fn new(btc_tx: &BtcTransaction, tx_info: &BtcOnIntBtcTxInfo, btc_account_nonce: u64) -> Result<IntOutput> {
        Ok(IntOutput {
            btc_account_nonce,
            btc_tx_hash: btc_tx.txid().to_string(),
            btc_tx_amount: tx_info.amount_in_satoshis,
            btc_tx_hex: get_hex_tx_from_signed_btc_tx(btc_tx),
            btc_tx_recipient: tx_info.recipient.clone(),
            originating_address: format!("0x{}", hex::encode(tx_info.from.as_bytes())),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash.as_bytes())),
            signature_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct EthOutput {
    pub int_latest_block_number: usize,
    pub btc_signed_transactions: Vec<IntOutput>,
}

pub fn get_btc_signed_tx_info_from_btc_txs(
    btc_account_nonce: u64,
    btc_txs: Vec<BtcTransaction>,
    redeem_infos: &BtcOnIntBtcTxInfos,
) -> Result<Vec<IntOutput>> {
    info!("✔ Getting BTC tx info from {} BTC tx(s)...", btc_txs.len());
    let num_btc_txs = btc_txs.len();
    let num_redeem_infos = redeem_infos.len();
    if num_btc_txs > num_redeem_infos {
        // NOTE: There CAN be fewer such as in the case of txs being filtered out for amounts being too low.
        return Err(format!(
            "There are MORE txs than tx infos! Num BTC txs: {}, Num RedeemInfos: {}",
            num_btc_txs, num_redeem_infos
        )
        .into());
    };
    let start_nonce = btc_account_nonce - btc_txs.len() as u64;
    btc_txs
        .iter()
        .enumerate()
        .map(|(i, btc_tx)| IntOutput::new(btc_tx, &redeem_infos.0[i], start_nonce + i as u64))
        .collect::<Result<Vec<_>>>()
}

pub fn get_int_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting INT output json...");
    let output = serde_json::to_string(&EthOutput {
        int_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_usize(),
        btc_signed_transactions: match state.btc_transactions {
            Some(txs) => get_btc_signed_tx_info_from_btc_txs(
                state.btc_db_utils.get_btc_account_nonce_from_db()?,
                txs,
                &state.btc_on_int_btc_tx_infos,
            )?,
            None => vec![],
        },
    })?;
    info!("✔ INT Output: {}", output);
    Ok(output)
}
