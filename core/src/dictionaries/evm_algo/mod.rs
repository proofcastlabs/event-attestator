pub(crate) mod dictionary;
pub(crate) mod dictionary_entry;
pub use crate::dictionaries::evm_algo::{
    dictionary::EvmAlgoTokenDictionary,
    dictionary_entry::EvmAlgoTokenDictionaryEntry,
};
use crate::{chains::algo::algo_state::AlgoState, state::EthState, traits::DatabaseInterface, types::Result};

pub fn get_evm_algo_token_dictionary_and_add_to_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Getting `EvmAlgoTokenDictionary` and adding to ETH state...");
    EvmAlgoTokenDictionary::get_from_db(state.eth_db_utils.get_db())
        .and_then(|dictionary| state.add_evm_algo_dictionary(dictionary))
}

pub fn get_evm_algo_token_dictionary_and_add_to_algo_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Getting `EvmAlgoTokenDictionary` and adding to ALGO state...");
    EvmAlgoTokenDictionary::get_from_db(state.algo_db_utils.get_db())
        .and_then(|dictionary| state.add_evm_algo_dictionary(dictionary))
}
