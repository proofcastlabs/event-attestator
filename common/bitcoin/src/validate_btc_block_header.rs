use common::{traits::DatabaseInterface, types::Result};

use crate::{btc_block::BtcBlockAndId, BtcState};

fn validate_btc_block_header(btc_block_and_id: &BtcBlockAndId) -> Result<()> {
    if btc_block_and_id.block.block_hash() == btc_block_and_id.id {
        info!("✔ BTC block header valid!");
        Ok(())
    } else {
        Err("✘ Invalid BTC block! Block header hash does not match block id!".into())
    }
}

pub fn validate_btc_block_header_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    if cfg!(feature = "non-validating") {
        info!("✔ Skipping BTC block-header validation!");
        Ok(state)
    } else {
        info!("✔ Validating BTC block header...");
        validate_btc_block_header(state.get_btc_block_and_id()?).map(|_| state)
    }
}

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use std::str::FromStr;

    use common::errors::AppError;

    use super::*;
    use crate::{
        bitcoin_crate_alias::BlockHash,
        btc_block::BtcBlockAndId,
        deposit_address_info::DepositInfoList,
        test_utils::get_sample_btc_block_and_id,
    };

    #[test]
    fn should_validate_btc_block_header() {
        let block_and_id = get_sample_btc_block_and_id().unwrap();
        if let Err(e) = validate_btc_block_header(&block_and_id) {
            panic!("Sample block should be valid: {}", e);
        }
    }

    #[test]
    fn should_error_on_invalid_block() {
        let expected_error = "✘ Invalid BTC block! Block header hash does not match block id!".to_string();
        let block_and_id = get_sample_btc_block_and_id().unwrap();
        let wrong_block_id = "c0ffee0000000000000c084f2a5fa68ef814144d350a601688248b421258dd3f";
        let invalid_block_and_id = BtcBlockAndId {
            height: 1,
            block: block_and_id.block,
            deposit_address_list: DepositInfoList::new(vec![]),
            id: BlockHash::from_str(wrong_block_id).unwrap(),
        };
        match validate_btc_block_header(&invalid_block_and_id) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Should not be valid!"),
            _ => panic!("Wrong error for invalid btc block!"),
        }
    }
}
