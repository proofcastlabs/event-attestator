use common::{state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn add_relevant_validated_txs_to_submission_material_in_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Replacing transactions in block in submission material with relevant, validated ones!");
    state
        .get_algo_submission_material()
        .and_then(|ref mut mutable_submission_material| {
            let txs = state.get_relevant_asset_txs()?.to_transactions();
            if txs.is_empty() {
                mutable_submission_material.block.transactions = None
            } else {
                mutable_submission_material.block.transactions = Some(txs)
            };
            state.update_algo_submission_material(mutable_submission_material)
        })
}
