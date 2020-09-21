use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chains::eos::filter_action_proofs::filter_out_already_processed_txs,
};

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out already processed tx IDs...");
    filter_out_already_processed_txs(&state.redeem_params, &state.processed_tx_ids)
        .and_then(|filtered_params| state.add_redeem_params(filtered_params))
}
