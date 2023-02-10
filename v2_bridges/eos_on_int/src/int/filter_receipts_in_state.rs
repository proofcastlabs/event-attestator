use common::{traits::DatabaseInterface, types::Result};
use common_eth::{EthState, ERC777_REDEEM_EVENT_TOPIC_V2};

pub fn filter_receipts_for_eos_on_int_eos_tx_info_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `EosOnIntEosTxInfo`...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_addresses_and_with_topics(
            &state.get_eos_eth_token_dictionary()?.to_eth_addresses(),
            &[*ERC777_REDEEM_EVENT_TOPIC_V2],
        )
        .and_then(|filtered_submission_material| state.update_eth_submission_material(filtered_submission_material))
}
