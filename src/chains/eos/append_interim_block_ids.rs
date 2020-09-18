use crate::{
    types::Result,
    chains::eos::{
        eos_types::Checksum256,
        eos_merkle_utils::Incremerkle,
    },
};

pub fn append_block_ids_to_incremerkle(
    mut incremerkle: Incremerkle,
    block_ids: &[Checksum256],
) -> Result<Incremerkle> {
    block_ids.iter().map(|id| incremerkle.append(*id)).for_each(drop);
    Ok(incremerkle)
}
