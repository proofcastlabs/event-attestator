use rust_algorand::{AlgorandBlock, AlgorandHash};

use crate::types::Result;

pub fn maybe_get_new_candidate_block_hash(
    current_block: &AlgorandBlock,
    maybe_candidate_block: Option<AlgorandBlock>,
) -> Result<Option<AlgorandHash>> {
    match maybe_candidate_block {
        None => {
            info!("✔ No candidate block in db yet ∴ not updating block hash!");
            Ok(None)
        },
        Some(candidate_block) => {
            info!("✔ Candidate block found!");
            if current_block.round() < candidate_block.round() {
                info!("✔ Current block IS older than new candidate block, ∴ updating it...");
                Ok(Some(candidate_block.hash()?))
            } else {
                info!("✘ Current block is NOT older than new candidate block ∴ NOT updating it!");
                Ok(None)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::algo::test_utils::get_sample_contiguous_blocks;

    #[test]
    fn should_get_candidate_block_hash_if_newer() {
        let blocks = get_sample_contiguous_blocks();
        let current_block = blocks[0].clone();
        let candidate_block = blocks[1].clone();
        let expected_result = Some(candidate_block.hash().unwrap());
        let result = maybe_get_new_candidate_block_hash(&current_block, Some(candidate_block)).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_get_candidate_block_hash_if_not_newer() {
        let blocks = get_sample_contiguous_blocks();
        let current_block = blocks[1].clone();
        let candidate_block = blocks[0].clone();
        let expected_result = None;
        let result = maybe_get_new_candidate_block_hash(&current_block, Some(candidate_block)).unwrap();
        assert_eq!(result, expected_result);
    }
}
