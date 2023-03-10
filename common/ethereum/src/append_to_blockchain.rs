use common::{traits::DatabaseInterface, types::Result};

use crate::{
    add_block_and_receipts_to_db::add_block_and_receipts_to_db_if_not_extant,
    check_parent_exists::check_for_parent_of_block,
    eth_database_utils::EthDbUtilsExt,
    eth_submission_material::EthSubmissionMaterial as EthSubMat,
    remove_old_eth_tail_block::maybe_remove_tail_and_older_blocks,
    update_eth_canon_block_hash::maybe_update_canon_block_hash,
    update_eth_linker_hash::maybe_update_linker_hash,
    update_eth_tail_block_hash::maybe_update_eth_tail_block_hash,
    update_latest_block_hash::update_latest_block_hash_if_subsequent,
};

/// Append to blockchain
///
/// A helper function that takes the db utils as an argument and appends the passed in submission
/// material to the chain in the database, if and only if it's subsequent.
///
/// WARN: No checks are done on the receipt's validity, since at this point they'll likely have
/// bene truncated down to only pertinent ones in order to save space in the database.
pub fn append_to_blockchain<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    sub_mat: &EthSubMat,
) -> Result<()> {
    let n = sub_mat.get_block_number()?;
    let chain_id = db_utils.get_eth_chain_id_from_db()?;
    let confs = db_utils.get_eth_canon_to_tip_length_from_db()?;

    if sub_mat.block_header_is_valid(&chain_id) {
        info!("Adding block {n} to chain in db...");
        check_for_parent_of_block(db_utils, sub_mat)
            .and_then(|_| add_block_and_receipts_to_db_if_not_extant(db_utils, sub_mat))
            .and_then(|_| update_latest_block_hash_if_subsequent(db_utils, sub_mat))
            .and_then(|_| maybe_update_canon_block_hash(db_utils, confs))
            .and_then(|_| maybe_update_eth_tail_block_hash(db_utils))
            .and_then(|_| maybe_update_linker_hash(db_utils))
            .and_then(|_| maybe_remove_tail_and_older_blocks(db_utils))
            .and(Ok(()))
    } else {
        Err("Not adding block to database because it's not valid!".into())
    }
}

// TODO test!
