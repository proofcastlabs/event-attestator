use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        filter_action_proofs::filter_out_already_processed_txs,
    },
};

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out already processed tx IDs...");
    filter_out_already_processed_txs(&state.btc_on_eos_redeem_infos, &state.processed_tx_ids)
        .and_then(|filtered| state.add_btc_on_eos_redeem_infos(filtered))
}
