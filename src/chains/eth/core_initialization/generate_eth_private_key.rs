use crate::{
    chains::eth::{eth_crypto::eth_private_key::EthPrivateKey, eth_state::EthState},
    traits::DatabaseInterface,
    types::Result,
};

pub fn generate_and_store_eth_private_key<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Generating & storing ETH private key...");
    state
        .eth_db_utils
        .put_eth_private_key_in_db(&EthPrivateKey::generate_random()?)
        .and(Ok(state))
}
