use bitcoin::network::constants::Network as BtcNetwork;

use crate::{chains::btc::btc_state::BtcState, traits::DatabaseInterface, types::Result};

pub fn put_btc_tail_block_hash_in_db_and_return_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    trace!("✔ Putting BTC tail block hash in db...");
    state
        .btc_db_utils
        .put_btc_tail_block_hash_in_db(&state.get_btc_block_and_id()?.id)
        .and(Ok(state))
}

pub fn put_btc_account_nonce_in_db_and_return_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    trace!("✔ Putting BTC account nonce of 0 in db...");
    state.btc_db_utils.put_btc_account_nonce_in_db(0).and(Ok(state))
}

pub fn put_canon_to_tip_length_in_db_and_return_state<D: DatabaseInterface>(
    canon_to_tip_length: u64,
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    state
        .btc_db_utils
        .put_btc_canon_to_tip_length_in_db(canon_to_tip_length)
        .and(Ok(state))
}

pub fn get_btc_network_from_arg(network_arg: &str) -> BtcNetwork {
    match network_arg {
        "Testnet" => {
            trace!("✔ Using 'TESTNET' for bitcoin network!");
            BtcNetwork::Testnet
        },
        _ => {
            trace!("✔ Using 'BITCOIN' for bitcoin network!");
            BtcNetwork::Bitcoin
        },
    }
}

pub fn put_difficulty_threshold_in_db<D: DatabaseInterface>(
    difficulty: u64,
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    state.btc_db_utils.put_btc_difficulty_in_db(difficulty).and(Ok(state))
}

pub fn put_btc_network_in_db_and_return_state<'a, D: DatabaseInterface>(
    network: &str,
    state: BtcState<'a, D>,
) -> Result<BtcState<'a, D>> {
    state
        .btc_db_utils
        .put_btc_network_in_db(get_btc_network_from_arg(network))
        .and(Ok(state))
}

pub fn put_btc_fee_in_db_and_return_state<D: DatabaseInterface>(fee: u64, state: BtcState<D>) -> Result<BtcState<D>> {
    state.btc_db_utils.put_btc_fee_in_db(fee).and(Ok(state))
}
