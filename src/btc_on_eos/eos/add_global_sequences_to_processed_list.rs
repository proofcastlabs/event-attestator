use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        add_global_sequences_to_processed_list::add_tx_ids_to_processed_tx_ids,
    },
};

pub fn maybe_add_global_sequences_to_processed_list_and_return_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    match &state.redeem_infos.len() {
        0 => {
            info!("✔ No `global_sequences` to add to processed tx list!");
            Ok(state)
        }
        _ => {
            info!("✔ Adding `global_sequences` to processed tx list...");
            add_tx_ids_to_processed_tx_ids(&state.db, &state.redeem_infos, &state.processed_tx_ids).and(Ok(state))
        }
    }
}
