use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    chains::{eos::eos_crypto::eos_transaction::EosSignedTransaction, eth::eth_database_utils::EthDbUtilsExt},
    erc20_on_eos::eth::eos_tx_info::{Erc20OnEosEosTxInfo, Erc20OnEosEosTxInfos},
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EosTxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub eos_tx_amount: String,
    pub eth_tx_amount: String,
    pub eos_account_nonce: u64,
    pub eos_tx_recipient: String,
    pub eos_tx_signature: String,
    pub witnessed_timestamp: u64,
    pub eos_serialized_tx: String,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub eos_latest_block_number: u64,
    pub native_token_address: String,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

impl EosTxInfo {
    pub fn new(
        eos_tx: &EosSignedTransaction,
        eos_tx_info: &Erc20OnEosEosTxInfo,
        eos_account_nonce: u64,
        eos_latest_block_number: u64,
    ) -> Result<EosTxInfo> {
        Ok(EosTxInfo {
            broadcast: false,
            eos_account_nonce,
            eos_latest_block_number,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            eos_tx_amount: eos_tx.amount.to_string(),
            eos_tx_signature: eos_tx.signature.clone(),
            eos_tx_recipient: eos_tx.recipient.clone(),
            eos_serialized_tx: eos_tx.transaction.clone(),
            eth_tx_amount: eos_tx_info.token_amount.to_string(),
            _id: format!("perc20-on-eos-eos-{}", eos_account_nonce),
            host_token_address: eos_tx_info.eos_token_address.to_string(),
            originating_address: format!("0x{}", hex::encode(eos_tx_info.token_sender)),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            originating_tx_hash: format!("0x{}", hex::encode(eos_tx_info.originating_tx_hash)),
            native_token_address: format!("0x{}", hex::encode(eos_tx_info.eth_token_address)),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Erc20OnEosEthOutput {
    pub eth_latest_block_number: u64,
    pub eos_signed_transactions: Vec<EosTxInfo>,
}

pub fn get_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting `erc20-on-eos` ETH submission output json...");
    Ok(serde_json::to_string(&Erc20OnEosEthOutput {
        eth_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_u64(),
        eos_signed_transactions: match state.eos_transactions {
            None => vec![],
            Some(ref eos_txs) => {
                let start_nonce = state.eos_db_utils.get_eos_account_nonce_from_db()? - eos_txs.len() as u64;
                let infos = Erc20OnEosEosTxInfos::from_bytes(&state.tx_infos)?;
                eos_txs
                    .iter()
                    .enumerate()
                    .map(|(i, eos_tx)| {
                        EosTxInfo::new(
                            eos_tx,
                            &infos[i],
                            start_nonce + i as u64,
                            state.eos_db_utils.get_latest_eos_block_number()?,
                        )
                    })
                    .collect::<Result<Vec<EosTxInfo>>>()?
            },
        },
    })?)
}
