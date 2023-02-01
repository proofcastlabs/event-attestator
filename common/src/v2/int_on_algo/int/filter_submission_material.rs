use crate::{
    chains::eth::{eth_contracts::erc20_vault::ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2, eth_database_utils::EthDbUtilsExt},
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn filter_submission_material_for_peg_in_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `int-on-algo` peg in events...");
    let vault_address = state.eth_db_utils.get_int_on_algo_smart_contract_address()?;
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(&vault_address, &[*ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2])
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}
