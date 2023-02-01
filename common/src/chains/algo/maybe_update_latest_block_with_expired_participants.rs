use rust_algorand::AlgorandAddress;

use crate::{
    chains::algo::{algo_database_utils::AlgoDbUtils, algo_submission_material::AlgoSubmissionMaterial},
    state::AlgoState,
    traits::DatabaseInterface,
    types::Result,
};

fn submission_material_has_expired_participants(submission_material: &AlgoSubmissionMaterial) -> bool {
    submission_material.expired_participation_accounts.is_some()
}

fn update_latest_block_with_expired_participants<D: DatabaseInterface>(
    algo_db_utils: &AlgoDbUtils<D>,
    expired_participants: Option<Vec<AlgorandAddress>>,
) -> Result<()> {
    // NOTE: So this is to mitigate the issue where the javascript algosdk does not include the
    // expired participation accounts in block headers (https://github.com/algorand/js-algorand-sdk/issues/587)
    // When this happens, the enclave submissions begin to fail due to the `no parent` error,
    // because the block header calculation for the block is incorrect without this field. In that
    // case, the syncer gets the extra field (not via the JS sdk!) and adds it to the submission
    // material. So if it's present in the submission material, we can assume we need to add it to
    // the _current_ latest block in order for it's header hash calculation to be correct. And so
    // to do that: First we get the current latest block from the db and its hash...
    let mut latest_submission_material = algo_db_utils.get_latest_submission_material()?;
    let old_latest_block_hash = latest_submission_material.block.hash()?;
    // NOTE: Then we delete it from the database entirely...
    algo_db_utils.delete_submission_material_by_hash(&latest_submission_material.block.hash()?)?;
    // NOTE: Then we update the latest to include the expired participation accounts...
    latest_submission_material
        .block
        .block_header
        .expired_participation_accounts = expired_participants;
    // NOTE: Then we put that update submission material back in as the latest block. Now, when it
    // comes to checking if the currently-being-submitted block is subsequent to the latest one,
    // this "latest" should now have the correct header hash, which the currently-being-submitted
    // block's `previous_block_hash` points to.
    algo_db_utils.put_latest_submission_material_in_db(&latest_submission_material)?;
    // NOTE: And should the currently-being-submitted block _not_ be subsequent despite the above,
    // the problem must lie elsewhere. But no harm is done as submission will fail so no database
    // transaction will take place and this change to the latest block will never be written.

    // NOTE: Finally, now that a block hash has changed, we need to update any other of the
    // critical hashes used by the core that might have been the old header hash:
    let new_latest_block_hash = algo_db_utils.get_latest_block_hash()?;
    let tail_block_hash = algo_db_utils.get_tail_block_hash()?;
    let canon_block_hash = algo_db_utils.get_canon_block_hash()?;
    let anchor_block_hash = algo_db_utils.get_anchor_block_hash()?;
    let genesis_block_hash = algo_db_utils.get_genesis_block_hash()?;
    if tail_block_hash == old_latest_block_hash {
        algo_db_utils.put_tail_block_hash_in_db(&new_latest_block_hash)?;
    }
    if canon_block_hash == old_latest_block_hash {
        algo_db_utils.put_canon_block_hash_in_db(&new_latest_block_hash)?;
    }
    if anchor_block_hash == old_latest_block_hash {
        algo_db_utils.put_anchor_block_hash_in_db(&new_latest_block_hash)?;
    }
    if genesis_block_hash == old_latest_block_hash {
        algo_db_utils.put_genesis_block_hash_in_db(&new_latest_block_hash)?;
    }

    Ok(())
}

pub fn maybe_update_latest_block_with_expired_participants_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    let submission_material = state.get_algo_submission_material()?;
    if submission_material_has_expired_participants(&submission_material) {
        info!("✔ Updating expired participation accounts in latest block...");
        update_latest_block_with_expired_participants(
            &state.algo_db_utils,
            submission_material.expired_participation_accounts,
        )
        .and(Ok(state))
    } else {
        info!("✘ No need to update expired participation accounts in latest block!");
        Ok(state)
    }
}
