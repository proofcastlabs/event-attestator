use crate::{chains::eth::eth_state::EthState, traits::DatabaseInterface, types::Result};

pub fn generate_and_store_eth_address<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Generating ETH address...");
    state
        .eth_db_utils
        .get_eth_private_key_from_db()
        .map(|pk| pk.to_public_key().to_address())
        .and_then(|address| state.eth_db_utils.put_public_eth_address_in_db(&address))
        .and(Ok(state))
}
