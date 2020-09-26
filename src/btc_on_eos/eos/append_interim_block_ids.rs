use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        append_interim_block_ids::append_block_ids_to_incremerkle,
    },
};

pub fn append_interim_block_ids_to_incremerkle_in_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Appending interim block IDs to incremerkle...");
    append_block_ids_to_incremerkle(state.incremerkle.clone(), &state.interim_block_ids)
        .map(|incremerkle| state.add_incremerkle(incremerkle))
}
