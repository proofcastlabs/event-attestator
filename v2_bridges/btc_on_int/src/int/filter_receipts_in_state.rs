use common::{traits::DatabaseInterface, types::Result};
use common_eth::{EthDbUtilsExt, EthState, ERC777_REDEEM_EVENT_TOPIC_V2};

pub fn filter_receipts_for_btc_on_int_redeem_events_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Filtering receipts for those containing `btc-on-eth` redeem events...");
    state
        .get_eth_submission_material()?
        .get_receipts_containing_log_from_address_and_with_topics(
            &state.eth_db_utils.get_btc_on_int_smart_contract_address_from_db()?,
            &[*ERC777_REDEEM_EVENT_TOPIC_V2],
        )
        .and_then(|filtered_block_and_receipts| state.update_eth_submission_material(filtered_block_and_receipts))
}
