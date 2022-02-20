#[cfg(test)]
use std::str::FromStr;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::errors::AppError;
use crate::{
    btc_on_int::btc::int_tx_info::BtcOnIntIntTxInfo,
    chains::{
        btc::{btc_constants::PLACEHOLDER_BTC_ADDRESS, btc_state::BtcState},
        eth::{
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::EthDbUtilsExt,
            eth_traits::EthTxInfoCompatible,
        },
    },
    traits::DatabaseInterface,
    types::Result,
    utils::get_unix_timestamp,
};

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Constructor)]
pub struct BtcOutput {
    pub btc_latest_block_number: u64,
    pub int_signed_transactions: Vec<IntTxInfo>,
}

#[cfg(test)]
impl FromStr for BtcOutput {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        #[derive(Serialize, Deserialize)]
        struct Interim {
            btc_latest_block_number: u64,
            int_signed_transactions: Vec<String>,
        }
        let interim = serde_json::from_str::<Interim>(s)?;
        let tx_infos = interim
            .int_signed_transactions
            .iter()
            .map(|inner_s| IntTxInfo::from_str(&inner_s))
            .collect::<Result<Vec<IntTxInfo>>>()?;
        Ok(Self {
            btc_latest_block_number: interim.btc_latest_block_number,
            int_signed_transactions: tx_infos,
        })
    }
}

// FIXME This needs standardizing with more recent cores!
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct IntTxInfo {
    pub int_tx_hex: Option<String>,
    pub int_tx_hash: String,
    pub int_tx_amount: String,
    pub int_account_nonce: Option<u64>,
    pub int_tx_recipient: String,
    pub signature_timestamp: u64,
    pub originating_tx_hash: String,
    pub originating_address: String,
}

#[cfg(test)]
impl FromStr for IntTxInfo {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

impl IntTxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        int_tx_info: &BtcOnIntIntTxInfo,
        nonce: Option<u64>,
    ) -> Result<IntTxInfo> {
        let default_address = PLACEHOLDER_BTC_ADDRESS.to_string();
        let retrieved_address = int_tx_info.originating_tx_address.to_string();
        let address_string = match default_address == retrieved_address {
            false => retrieved_address,
            true => "✘ Could not retrieve sender address".to_string(),
        };

        Ok(IntTxInfo {
            int_account_nonce: nonce,
            int_tx_hex: tx.eth_tx_hex(),
            originating_address: address_string,
            signature_timestamp: get_unix_timestamp()?,
            int_tx_amount: int_tx_info.amount.to_string(),
            int_tx_hash: format!("0x{}", tx.get_tx_hash()),
            originating_tx_hash: int_tx_info.originating_tx_hash.to_string(),
            int_tx_recipient: format!("0x{}", hex::encode(int_tx_info.destination_address.as_bytes())),
        })
    }
}

pub fn get_eth_signed_tx_info_from_eth_txs(
    int_txs: &[EthTransaction],
    int_tx_infos: &[BtcOnIntIntTxInfo],
    int_account_nonce: u64,
) -> Result<Vec<IntTxInfo>> {
    info!("✔ Getting INT tx info from INT txs...");
    let number_of_txs = int_txs.len() as u64;
    let start_nonce = if number_of_txs > int_account_nonce {
        return Err("INT account nonce has not been incremented correctly!".into());
    } else {
        int_account_nonce - number_of_txs
    };
    let start_nonce = int_account_nonce - int_txs.len() as u64;
    int_txs
        .iter()
        .enumerate()
        .map(|(i, tx)| IntTxInfo::new(tx, &int_tx_infos[i], Some(start_nonce + i as u64)))
        .collect::<Result<Vec<IntTxInfo>>>()
}

pub fn get_btc_output_and_put_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Getting BTC output json and putting in state...");
    let signed_txs = state.eth_signed_txs.clone();
    let output = BtcOutput {
        btc_latest_block_number: state.btc_db_utils.get_btc_latest_block_from_db()?.height,
        int_signed_transactions: if signed_txs.len() == 0 {
            vec![]
        } else {
            get_eth_signed_tx_info_from_eth_txs(
                &state.eth_signed_txs,
                &state
                    .btc_db_utils
                    .get_btc_canon_block_from_db()?
                    .get_btc_on_int_int_tx_infos(),
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
            )?
        },
    };
    state.add_output_json_string(serde_json::to_string(&output)?)
}

pub fn get_btc_output_as_string<D: DatabaseInterface>(state: BtcState<D>) -> Result<String> {
    state.get_output_json_string()
}
