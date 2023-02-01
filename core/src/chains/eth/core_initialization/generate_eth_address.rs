use crate::{
    chains::eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    traits::DatabaseInterface,
    types::Result,
};

fn generate_and_store_address<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<()> {
    db_utils
        .get_eth_private_key_from_db()
        .map(|pk| pk.to_public_key().to_address())
        .and_then(|ref address| db_utils.put_public_eth_address_in_db(address))
}

pub fn generate_and_store_eth_address<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Generating & storing ETH address...");
    generate_and_store_address(&state.eth_db_utils).and(Ok(state))
}

pub fn generate_and_store_evm_address<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Generating & storing EVM address...");
    generate_and_store_address(&state.evm_db_utils).and(Ok(state))
}
