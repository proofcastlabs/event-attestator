use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::eth::{
        eth_constants::PTOKEN_CONTRACT_TOPICS,
        eth_database_utils::get_erc777_contract_address_from_db,
    },
};

pub fn filter_irrelevant_receipts_from_state<D>(state: EthState<D>) -> Result<EthState<D>> where D: DatabaseInterface {
    info!("✔ Filtering out non-pToken related receipts...");
    state
        .get_eth_block_and_receipts()?
        .filter_for_receipts_containing_log_with_address_and_topics(
            &get_erc777_contract_address_from_db(&state.db)?,
            &PTOKEN_CONTRACT_TOPICS.to_vec(),
        )
        .and_then(|filtered_block_and_receipts| state.update_eth_block_and_receipts(filtered_block_and_receipts))
}
