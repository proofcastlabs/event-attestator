use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::eth_database_utils::key_exists_in_db,
    btc_on_eth::{
        eth::eth_state::EthState,
        utils::convert_h256_to_bytes,
    },
};

pub fn check_for_parent_of_block_in_state<D>(state: EthState<D>) -> Result<EthState<D>> where D: DatabaseInterface {
    info!("✔ Checking block's parent exists in database...");
    match key_exists_in_db(&state.db, &convert_h256_to_bytes(state.get_parent_hash()?),  None) {
        true => {
            info!("✔ Block's parent exists in database!");
            Ok(state)
        },
        false => Err("✘ Block Rejected - no parent exists in database!".into()),
    }
}
