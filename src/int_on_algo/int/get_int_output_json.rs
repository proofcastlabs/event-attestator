use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    chains::{
        algo::algo_signed_group_txs::{AlgoSignedGroupTx, AlgoSignedGroupTxs},
        eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState, eth_utils::convert_eth_address_to_string},
    },
    int_on_algo::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

make_output_structs!(Int, Algo);

make_struct_with_test_assertions_on_equality_check!(
    struct AlgoTxInfo {
        _id: String,
        broadcast: bool,
        algo_tx_hash: String,
        algo_signed_tx: String,
        algo_tx_amount: String,
        algo_account_nonce: u64,
        witnessed_timestamp: u64,
        algo_tx_recipient: String,
        host_token_address: String,
        originating_tx_hash: String,
        originating_address: String,
        native_token_address: String,
        destination_chain_id: String,
        algo_latest_block_number: u64,
        broadcast_tx_hash: Option<String>,
        broadcast_timestamp: Option<String>,
    }
);

impl AlgoTxInfo {
    pub fn new(
        group_tx: AlgoSignedGroupTx,
        tx_info: &IntOnAlgoAlgoTxInfo,
        nonce: u64,
        algo_latest_block_number: u64,
    ) -> Result<AlgoTxInfo> {
        Ok(AlgoTxInfo {
            broadcast: false,
            broadcast_tx_hash: None,
            algo_latest_block_number,
            broadcast_timestamp: None,
            algo_account_nonce: nonce,
            algo_signed_tx: group_tx.signed_tx,
            algo_tx_hash: group_tx.group_tx.to_id()?,
            _id: format!("pint-on-algo-algo-{}", nonce),
            algo_tx_amount: tx_info.host_token_amount.to_string(),
            host_token_address: format!("{}", tx_info.algo_asset_id),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            native_token_address: format!("0x{}", hex::encode(tx_info.int_token_address)),
            originating_address: convert_eth_address_to_string(&tx_info.token_sender.clone()),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash.as_bytes())),
            destination_chain_id: format!("0x{}", hex::encode(&tx_info.destination_chain_id.to_bytes()?)),
            algo_tx_recipient: if tx_info.destination_is_app() {
                tx_info.get_destination_app_id()?.to_string()
            } else {
                tx_info.get_destination_address()?.to_string()
            },
        })
    }
}

pub fn get_int_signed_tx_info_from_int_txs(
    txs: AlgoSignedGroupTxs,
    tx_infos: &IntOnAlgoAlgoTxInfos,
    algo_account_nonce: u64,
    algo_latest_block_num: u64,
) -> Result<Vec<AlgoTxInfo>> {
    let number_of_txs = txs.len() as u64;
    let start_nonce = algo_account_nonce - number_of_txs;
    info!("✔ Getting INT tx info from ALGO txs...");
    if number_of_txs > algo_account_nonce {
        return Err("ALGO account nonce has not been incremented correctly!".into());
    };
    if number_of_txs != tx_infos.len() as u64 {
        return Err("Number of txs does not match number of tx infos!".into());
    };
    txs.iter()
        .zip(tx_infos.iter())
        .enumerate()
        .map(|(i, (tx, info))| AlgoTxInfo::new(tx.clone(), info, start_nonce + i as u64, algo_latest_block_num))
        .collect::<Result<Vec<_>>>()
}

pub fn get_int_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<IntOutput> {
    info!("✔ Getting INT output json...");
    let txs = state.algo_signed_group_txs.clone();
    let int_latest_block_num = state.eth_db_utils.get_latest_eth_block_number()?;
    let output = if !txs.is_empty() {
        IntOutput::new(
            int_latest_block_num,
            get_int_signed_tx_info_from_int_txs(
                txs,
                &state.int_on_algo_algo_tx_infos,
                state.algo_db_utils.get_algo_account_nonce()?,
                state.algo_db_utils.get_latest_block_number()?,
            )?,
        )
    } else {
        IntOutput::new(int_latest_block_num, vec![])
    };
    Ok(output)
}
