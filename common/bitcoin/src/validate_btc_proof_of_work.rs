use common::{traits::DatabaseInterface, types::Result};

use crate::{bitcoin_crate_alias::blockdata::block::BlockHeader as BtcBlockHeader, BtcState};

fn validate_proof_of_work_in_block(btc_block_header: &BtcBlockHeader) -> Result<()> {
    match btc_block_header.validate_pow(&btc_block_header.target()) {
        Ok(_) => {
            info!("✔ BTC block's proof-of-work is valid!");
            Ok(())
        },
        Err(_) => Err("✘ Invalid block! PoW validation error: Block hash > target!".into()),
    }
}

pub fn validate_proof_of_work_of_btc_block_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    if cfg!(feature = "non-validating") {
        info!("✔ Skipping BTC proof-of-work validation!");
        Ok(state)
    } else {
        info!("✔ Validating BTC block's proof-of-work...");
        validate_proof_of_work_in_block(&state.get_btc_block_and_id()?.block.header).map(|_| state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_btc_block_and_id;

    #[test]
    fn should_validate_proof_of_work_in_valid_block() {
        let block_header = get_sample_btc_block_and_id().unwrap().block.header;
        if let Err(e) = validate_proof_of_work_in_block(&block_header) {
            panic!("PoW should be valid in sample block: {}", e);
        }
    }
}
