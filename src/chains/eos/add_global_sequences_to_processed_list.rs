use crate::{
    chains::eos::{
        eos_database_utils::put_processed_tx_ids_in_db,
        eos_state::EosState,
        eos_types::{GlobalSequences, ProcessedTxIds},
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn add_tx_ids_to_processed_tx_ids<D: DatabaseInterface>(
    db: &D,
    processed_tx_ids: &ProcessedTxIds,
    mut global_sequences: GlobalSequences,
) -> Result<()> {
    put_processed_tx_ids_in_db(db, &processed_tx_ids.clone().add_multi(&mut global_sequences)?)
}

pub fn maybe_add_global_sequences_to_processed_list_and_return_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    let global_sequences = state.get_global_sequences();
    match global_sequences.len() {
        0 => {
            info!("✔ No `global_sequences` to add to processed tx list!");
            Ok(state)
        },
        _ => {
            info!("✔ Adding `global_sequences` to processed tx list...");
            add_tx_ids_to_processed_tx_ids(&state.db, &state.processed_tx_ids, global_sequences).and(Ok(state))
        },
    }
}
