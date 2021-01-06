use crate::{
    chains::eth::{
        eth_constants::ERC20_ON_EOS_PEG_IN_EVENT_TOPIC,
        eth_database_utils::get_erc20_on_eos_smart_contract_address_from_db,
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn filter_receipts_for_erc20_on_eos_peg_in_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `erc20-on-eos` peg in events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(
            &get_erc20_on_eos_smart_contract_address_from_db(&state.db)?,
            &ERC20_ON_EOS_PEG_IN_EVENT_TOPIC.to_vec(),
        )
        .and_then(|filtered| {
            filtered.filter_receipts_containing_supported_erc20_peg_ins(state.get_eos_erc20_dictionary()?)
        })
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}
