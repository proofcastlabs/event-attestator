use crate::{
    chains::eth::{eth_crypto::eth_private_key::EthPrivateKey, eth_database_utils::EthDbUtilsExt, EthState},
    traits::DatabaseInterface,
    types::Result,
};

fn generate_and_store_private_key<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<()> {
    db_utils.put_eth_private_key_in_db(&EthPrivateKey::generate_random()?)
}

pub fn generate_and_store_eth_private_key<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Generating & storing ETH private key...");
    generate_and_store_private_key(&state.eth_db_utils).and(Ok(state))
}

pub fn generate_and_store_evm_private_key<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Generating & storing EVM private key...");
    generate_and_store_private_key(&state.evm_db_utils).and(Ok(state))
}
