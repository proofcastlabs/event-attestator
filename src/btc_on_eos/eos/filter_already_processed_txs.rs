use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chains::eos::eos_types::{
        RedeemInfo,
        ProcessedTxIds,
    },
};

fn filter_out_already_processed_txs(
    redeem_params: &[RedeemInfo],
    processed_tx_ids: &ProcessedTxIds,
) -> Result<Vec<RedeemInfo>> {
    Ok(
        redeem_params
            .iter()
            .filter(|params| !processed_tx_ids.contains(&params.global_sequence))
            .cloned()
            .collect::<Vec<RedeemInfo>>()
    )
}

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out already processed tx IDs...");
    filter_out_already_processed_txs(
        &state.redeem_params,
        &state.processed_tx_ids,
    )
        .and_then(|filtered_params| state.add_redeem_params(filtered_params))
}
