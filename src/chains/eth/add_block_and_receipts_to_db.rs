use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn add_block_and_receipts_to_db_if_not_extant<D: DatabaseInterface>(
    eth_db_utils: &EthDbUtils<D>,
    block_and_receipts: &EthSubmissionMaterial,
) -> Result<()> {
    info!("✔ Adding ETH block and receipts if not already in db...");
    match eth_db_utils.eth_block_exists_in_db(&block_and_receipts.get_block_hash()?) {
        false => {
            info!("✔ Block & receipts not in db, adding them now...");
            eth_db_utils.put_eth_submission_material_in_db(block_and_receipts)
        },
        true => Err("✘ Block Rejected - it's already in the db!".into()),
    }
}

fn maybe_add_block_and_receipts_to_db_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe adding ETH block and receipts if not in db...");
    add_block_and_receipts_to_db_if_not_extant(
        if is_for_eth {
            &state.eth_db_utils
        } else {
            &state.evm_db_utils
        },
        state.get_eth_submission_material()?,
    )
    .and(Ok(state))
}

pub fn maybe_add_eth_block_and_receipts_to_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_add_block_and_receipts_to_db_and_return_state(true, state)
}

pub fn maybe_add_evm_block_and_receipts_to_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_add_block_and_receipts_to_db_and_return_state(false, state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::eth::eth_test_utils::get_sample_eth_submission_material_n, test_utils::get_test_database};

    #[test]
    fn should_maybe_add_block_and_receipts_to_db() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new_for_eth(&db);
        let block_and_receipts = get_sample_eth_submission_material_n(1).unwrap();
        let eth_block_hash = block_and_receipts.get_block_hash().unwrap();
        let bool_before = eth_db_utils.eth_block_exists_in_db(&eth_block_hash);
        assert!(!bool_before);
        add_block_and_receipts_to_db_if_not_extant(&eth_db_utils, &block_and_receipts).unwrap();
        let bool_after = eth_db_utils.eth_block_exists_in_db(&eth_block_hash);
        assert!(bool_after);
    }

    #[test]
    fn should_error_if_block_already_in_db() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new_for_eth(&db);
        let block_and_receipts = get_sample_eth_submission_material_n(1).unwrap();
        let eth_block_hash = block_and_receipts.get_block_hash().unwrap();
        let bool_before = eth_db_utils.eth_block_exists_in_db(&eth_block_hash);
        assert!(!bool_before);
        add_block_and_receipts_to_db_if_not_extant(&eth_db_utils, &block_and_receipts).unwrap();
        let bool_after = eth_db_utils.eth_block_exists_in_db(&eth_block_hash);
        assert!(add_block_and_receipts_to_db_if_not_extant(&eth_db_utils, &block_and_receipts).is_err());
        assert!(bool_after);
    }
}
