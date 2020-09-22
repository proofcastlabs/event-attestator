use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::{
        eth_state::EthState,
        eth_constants::PTOKEN_CONTRACT_TOPICS,
        eth_database_utils::get_erc777_contract_address_from_db,
    },
};

pub fn filter_irrelevant_receipts_from_state<D>(state: EthState<D>) -> Result<EthState<D>> where D: DatabaseInterface {
    info!("✔ Filtering out non-pToken related receipts...");
    state
        .get_eth_submission_material()?
        .filter_for_receipts_containing_log_with_address_and_topics(
            &get_erc777_contract_address_from_db(&state.db)?,
            &PTOKEN_CONTRACT_TOPICS.to_vec(),
        )
        .and_then(|filtered_block_and_receipts| state.update_eth_submission_material(filtered_block_and_receipts))
}
