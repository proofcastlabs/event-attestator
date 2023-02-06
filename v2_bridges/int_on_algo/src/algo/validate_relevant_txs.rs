use common::{state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn filter_out_invalid_txs_and_update_in_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    info!("✔ Validating relevant Algo asset txs and updating in state...");
    let txs = state.get_relevant_asset_txs()?;
    debug!("Number of relevant Algo txs before: {}", txs.len());
    let filtered_txs = txs.filter_out_invalid_txs(&state.get_algo_submission_material()?);
    debug!("Number of relevant Algo txs before: {}", filtered_txs.len());
    state.update_relevant_asset_txs(&filtered_txs)
}
