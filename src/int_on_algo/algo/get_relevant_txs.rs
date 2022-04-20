use derive_more::{Constructor, Deref};
use rust_algorand::{AlgorandAddress, AlgorandTransaction, AlgorandTransactionProof, AlgorandTransactions};

use crate::{
    chains::algo::{
        algo_relevant_asset_txs::{AlgoRelevantAssetTx, AlgoRelevantAssetTxs},
        algo_state::AlgoState,
        algo_submission_material::AlgoSubmissionMaterial,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_relevant_asset_txs_from_submission_material_and_add_to_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Getting relevant Algo asset transfer txs and adding to state...");
    AlgoRelevantAssetTxs::from_submission_material_for_assets_and_receivers(
        &state.get_algo_submission_material()?,
        state.get_evm_algo_token_dictionary()?.to_algo_asset_ids(),
        vec![state.algo_db_utils.get_redeem_address()?],
    )
    .and_then(|ref relevant_txs| state.add_relevant_asset_txs(relevant_txs))
}
