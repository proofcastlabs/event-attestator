use std::time::{SystemTime, UNIX_EPOCH};

use common::{
    chains::eos::{eos_chain_id::EosChainId, eos_crypto::eos_transaction::EosSignedTransaction},
    metadata::ToMetadataChainId,
    traits::DatabaseInterface,
    types::Result,
};
use common_eth::{EthDbUtilsExt, EthState};

use crate::int::{EosOnIntEosTxInfo, EosOnIntEosTxInfos};

make_output_structs!(Int, Eos);

make_struct_with_test_assertions_on_equality_check!(
    struct EosTxInfo {
        _id: String,
        broadcast: bool,
        eos_tx_amount: String,
        int_tx_amount: String,
        eos_account_nonce: u64,
        eos_tx_recipient: String,
        eos_tx_signature: String,
        witnessed_timestamp: u64,
        eos_serialized_tx: String,
        host_token_address: String,
        originating_tx_hash: String,
        originating_address: String,
        eos_latest_block_number: u64,
        native_token_address: String,
        destination_chain_id: String,
        broadcast_tx_hash: Option<String>,
        broadcast_timestamp: Option<String>,
    }
);

impl EosTxInfo {
    pub fn new(
        eos_tx: &EosSignedTransaction,
        tx_info: &EosOnIntEosTxInfo,
        eos_account_nonce: u64,
        eos_latest_block_number: u64,
        eos_chain_id: &EosChainId,
    ) -> Result<EosTxInfo> {
        Ok(EosTxInfo {
            broadcast: false,
            eos_account_nonce,
            eos_latest_block_number,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            eos_tx_signature: eos_tx.signature.clone(),
            eos_tx_recipient: eos_tx.recipient.clone(),
            eos_serialized_tx: eos_tx.transaction.clone(),
            int_tx_amount: tx_info.token_amount.to_string(),
            eos_tx_amount: tx_info.eos_asset_amount.clone(),
            _id: format!("peos-on-int-eos-{}", eos_account_nonce),
            native_token_address: tx_info.eos_token_address.to_string(),
            destination_chain_id: eos_chain_id.to_metadata_chain_id().to_hex()?,
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender)),
            host_token_address: format!("0x{}", hex::encode(tx_info.int_token_address)),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash)),
        })
    }
}

pub fn get_int_output<D: DatabaseInterface>(state: EthState<D>) -> Result<IntOutput> {
    info!("✔ Getting `eos-on-int` INT submission output json...");
    Ok(IntOutput {
        int_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_usize(),
        eos_signed_transactions: match state.eos_transactions {
            None => vec![],
            Some(ref eos_txs) => {
                let number_of_txs = eos_txs.len() as u64;
                let eos_account_nonce = state.eos_db_utils.get_eos_account_nonce_from_db()?;
                let tx_infos = EosOnIntEosTxInfos::from_bytes(&state.tx_infos)?;
                let start_nonce = if number_of_txs > eos_account_nonce {
                    return Err("EOS account nonce has not been incremented correctly!".into());
                } else {
                    eos_account_nonce - number_of_txs
                };
                eos_txs
                    .iter()
                    .enumerate()
                    .map(|(i, eos_tx)| {
                        EosTxInfo::new(
                            eos_tx,
                            &tx_infos[i],
                            start_nonce + i as u64,
                            state.eos_db_utils.get_latest_eos_block_number()?,
                            &state.eos_db_utils.get_eos_chain_id_from_db()?,
                        )
                    })
                    .collect::<Result<Vec<EosTxInfo>>>()?
            },
        },
    })
}
