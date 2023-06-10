use common::{traits::DatabaseInterface, types::Result};

use crate::{bitcoin_crate_alias::blockdata::block::Block as BtcBlock, BtcState};

fn validate_merkle_root(btc_block: &BtcBlock) -> Result<()> {
    match btc_block.check_merkle_root() {
        true => {
            info!("✔ Merkle-root valid!");
            Ok(())
        },
        false => Err("✘ Invalid block! Merkle root doesn't match calculated merkle root!".into()),
    }
}

pub fn validate_btc_merkle_root<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    if cfg!(feature = "non-validating") {
        info!("✔ Skipping BTC merkle-root validation!");
        Ok(state)
    } else {
        info!("✔ Validating merkle-root in BTC block...");
        validate_merkle_root(&state.get_btc_block_and_id()?.block).map(|_| state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_btc_block_and_id;

    #[test]
    fn should_validate_sample_merkle_root() {
        let block = get_sample_btc_block_and_id().unwrap().block;
        if let Err(e) = validate_merkle_root(&block) {
            panic!("Merkle root should be valid for samle block: {}", e);
        }
    }
}
