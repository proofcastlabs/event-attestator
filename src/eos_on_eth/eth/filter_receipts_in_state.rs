use crate::{
    chains::eth::{
        eth_constants::EOS_ON_ETH_ETH_TX_INFO_EVENT_TOPIC,
        eth_database_utils::get_eos_on_eth_smart_contract_address_from_db,
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn filter_receipts_for_eos_on_eth_eth_tx_info_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `eos-on-eth` tx info...");
    state
        .get_eth_submission_material()?
        .filter_for_receipts_containing_log_with_address_and_topics(
            &get_eos_on_eth_smart_contract_address_from_db(&state.db)?,
            &EOS_ON_ETH_ETH_TX_INFO_EVENT_TOPIC.to_vec(),
        )
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}
