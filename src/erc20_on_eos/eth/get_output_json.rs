use std::time::{
    SystemTime,
    UNIX_EPOCH,
};
use crate::{
    types::Result,
    traits::DatabaseInterface,
    erc20_on_eos::eth::peg_in_info::Erc20OnEosPegInInfo,
    chains::{
        eth::{
            eth_state::EthState,
            eth_database_utils::get_eth_latest_block_from_db,
        },
        eos::{
            eos_types::EosSignedTransaction,
            eos_database_utils::{
                get_eos_account_nonce_from_db,
                get_latest_eos_block_number,
            },
        },
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EosTxInfo {
    pub broadcast: bool,
    pub eos_tx_amount: String,
    pub eos_account_nonce: u64,
    pub eos_tx_recipient: String,
    pub eos_tx_signature: String,
    pub witnessed_timestamp: u64,
    pub eos_serialized_tx: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub eos_latest_block_number: u64,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

impl EosTxInfo {
    pub fn new(
        eos_tx: &EosSignedTransaction,
        peg_in_info: &Erc20OnEosPegInInfo,
        eos_account_nonce: u64,
        eos_latest_block_number: u64,
    ) -> Result<EosTxInfo> {
        Ok(
            EosTxInfo {
                broadcast: false,
                eos_account_nonce,
                eos_latest_block_number,
                broadcast_tx_hash: None,
                broadcast_timestamp: None,
                eos_serialized_tx: eos_tx.transaction.clone(),
                eos_tx_amount: eos_tx.amount.to_string(),
                eos_tx_signature: eos_tx.signature.clone(),
                eos_tx_recipient: eos_tx.recipient.clone(),
                originating_address: peg_in_info.token_sender.to_string(),
                originating_tx_hash: peg_in_info.originating_tx_hash.to_string(),
                witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            }
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Erc20OnEosEthOutput {
    pub eth_latest_block_number: u64,
    pub eos_signed_transactions: Vec<EosTxInfo>,
}

pub fn get_output_json<D>(state: EthState<D>) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Getting `erc20-on-eos` ETH submission output json...");
    Ok(serde_json::to_string(
        &Erc20OnEosEthOutput {
            eth_latest_block_number: get_eth_latest_block_from_db(&state.db)?.block.number.as_u64(),
            eos_signed_transactions: match &state.eos_transactions {
                None => vec![],
                Some(eos_txs) => eos_txs
                    .iter()
                    .enumerate()
                    .map(|(i, eos_tx)| EosTxInfo::new(
                        &eos_tx,
                        &state.erc20_on_eos_peg_in_infos[i],
                        get_eos_account_nonce_from_db(&state.db)?,
                        get_latest_eos_block_number(&state.db)?,
                    ))
                    .collect::<Result<Vec<EosTxInfo>>>()?
            }
        }
    )?)
}
