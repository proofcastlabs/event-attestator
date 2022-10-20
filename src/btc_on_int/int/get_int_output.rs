use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::blockdata::transaction::Transaction as BtcTransaction;
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{
    btc_on_int::int::btc_tx_info::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
    chains::{
        btc::{btc_chain_id::BtcChainId, btc_utils::get_hex_tx_from_signed_btc_tx},
        eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    },
    traits::DatabaseInterface,
    types::Result,
};

make_plural_output_struct!(IntOutput, BtcTxInfo, btc_signed_transactions, int_latest_block_number);

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Constructor)]
pub struct IntOutput {
    pub int_latest_block_number: usize,
    pub btc_signed_transactions: Vec<BtcTxInfo>,
}

make_struct_with_test_assertions_on_equality_check!(
    struct BtcTxInfo {
        _id: String,
        broadcast: bool,
        btc_tx_hash: String,
        btc_tx_amount: u64,
        btc_signed_tx: String,
        btc_account_nonce: u64,
        witnessed_timestamp: u64,
        btc_tx_recipient: String,
        host_token_address: String,
        originating_address: String,
        originating_tx_hash: String,
        btc_latest_block_number: u64,
        destination_chain_id: String,
        broadcast_tx_hash: Option<String>,
        broadcast_timestamp: Option<usize>,
    }
);

impl BtcTxInfo {
    pub fn new(
        btc_tx: &BtcTransaction,
        tx_info: &BtcOnIntBtcTxInfo,
        btc_account_nonce: u64,
        btc_latest_block_number: u64,
        host_token_address: &EthAddress,
        btc_chain_id: &BtcChainId,
    ) -> Result<BtcTxInfo> {
        Ok(BtcTxInfo {
            broadcast: false,
            btc_account_nonce,
            broadcast_tx_hash: None,
            btc_latest_block_number,
            broadcast_timestamp: None,
            btc_tx_hash: btc_tx.txid().to_string(),
            btc_tx_amount: tx_info.amount_in_satoshis,
            btc_tx_recipient: tx_info.recipient.clone(),
            _id: format!("pbtc-on-int-btc-{btc_account_nonce}"),
            btc_signed_tx: get_hex_tx_from_signed_btc_tx(btc_tx),
            host_token_address: format!("0x{}", hex::encode(host_token_address)),
            originating_address: format!("0x{}", hex::encode(tx_info.from.as_bytes())),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash.as_bytes())),
            destination_chain_id: format!("0x{}", hex::encode(btc_chain_id.to_metadata_chain_id().to_bytes()?)),
        })
    }
}

pub fn get_btc_signed_tx_info_from_btc_txs(
    btc_account_nonce: u64,
    btc_txs: Vec<BtcTransaction>,
    redeem_infos: &BtcOnIntBtcTxInfos,
    btc_latest_block_number: u64,
    host_token_address: &EthAddress,
    btc_chain_id: &BtcChainId,
) -> Result<Vec<BtcTxInfo>> {
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
        .map(|(i, btc_tx)| {
            BtcTxInfo::new(
                btc_tx,
                &redeem_infos.0[i],
                start_nonce + i as u64,
                btc_latest_block_number,
                host_token_address,
                btc_chain_id,
            )
        })
        .collect::<Result<Vec<_>>>()
}

pub fn get_int_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<IntOutput> {
    info!("✔ Getting INT output json...");
    let output = IntOutput {
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
                state.btc_db_utils.get_latest_btc_block_number()?,
                &state.eth_db_utils.get_btc_on_int_smart_contract_address_from_db()?,
                &state.btc_db_utils.get_btc_chain_id_from_db()?,
            )?,
            None => vec![],
        },
    };
    info!("✔ INT Output: {}", output);
    Ok(output)
}
