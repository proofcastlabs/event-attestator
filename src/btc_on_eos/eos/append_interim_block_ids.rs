use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::{
        eos::{
            eos_state::EosState,
            eos_types::Checksum256s,
            eos_merkle_utils::IncreMerkle,
        },
    },
};

fn append_block_ids_to_incremerkle(
    mut incremerkle: IncreMerkle,
    block_ids: &Checksum256s,
) -> Result<IncreMerkle> {
    block_ids
        .iter()
        .map(|id| incremerkle.append(*id))
        .for_each(drop);
    Ok(incremerkle)
}

pub fn append_interim_block_ids_to_incremerkle_in_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Appending interim block IDs to incremerkle...");
    append_block_ids_to_incremerkle(
        state.incremerkle.clone(),
        &state.interim_block_ids,
    )
        .map(|incremerkle| state.add_incremerkle(incremerkle))
}
