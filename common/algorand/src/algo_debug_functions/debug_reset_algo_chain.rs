use common::{constants::ZERO_CONFS_WARNING, core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use rust_algorand::AlgorandHash;
use serde_json::json;

use crate::{
    add_latest_algo_submission_material_to_db_and_return_state,
    end_algo_db_transaction_and_return_state,
    initialize_algo_chain_db_keys,
    parse_algo_submission_material_and_put_in_state,
    remove_all_txs_from_submission_material_in_state,
    AlgoDbUtils,
    AlgoState,
};

fn delete_all_algo_submision_material<D: DatabaseInterface>(algo_db_utils: &AlgoDbUtils<D>) -> Result<()> {
    fn recursively_delete_all_algo_blocks<D: DatabaseInterface>(
        algo_db_utils: &AlgoDbUtils<D>,
        maybe_block_hash: Option<AlgorandHash>,
    ) -> Result<()> {
        match maybe_block_hash {
            None => {
                info!("✔ Deleting all ALGO blocks from db, starting with the latest block...");
                recursively_delete_all_algo_blocks(algo_db_utils, Some(algo_db_utils.get_latest_block_hash()?))
            },
            Some(ref hash) => match algo_db_utils.get_submission_material(hash) {
                Ok(material) => algo_db_utils
                    .delete_submission_material_by_hash(&material.block.hash()?)
                    .and_then(|_| {
                        recursively_delete_all_algo_blocks(
                            algo_db_utils,
                            Some(material.block.get_previous_block_hash()?),
                        )
                    }),
                Err(_) => {
                    info!("✔ All ALGO blocks deleted!");
                    Ok(())
                },
            },
        }
    }
    recursively_delete_all_algo_blocks(algo_db_utils, None)
}

fn delete_all_relevant_db_keys<D: DatabaseInterface>(algo_db_utils: &AlgoDbUtils<D>) -> Result<()> {
    vec![
        algo_db_utils.get_algo_linker_block_hash_key(),
        algo_db_utils.get_algo_canon_block_hash_key(),
        algo_db_utils.get_algo_tail_block_hash_key(),
        algo_db_utils.get_algo_anchor_block_hash_key(),
        algo_db_utils.get_algo_latest_block_hash_key(),
        algo_db_utils.get_algo_canon_to_tip_length_key(),
    ]
    .iter()
    .map(|key| algo_db_utils.get_db().delete(key.to_vec()))
    .collect::<Result<Vec<()>>>()
    .and(Ok(()))
}

fn delete_all_algo_submission_material_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    delete_all_algo_submision_material(&state.algo_db_utils)
        .and_then(|_| delete_all_relevant_db_keys(&state.algo_db_utils))
        .and(Ok(state))
}

pub fn reset_algo_chain_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
    canon_to_tip_length: u64,
) -> Result<AlgoState<D>> {
    info!("Resetting ALGO chain...");
    delete_all_algo_submission_material_and_return_state(state)
        .and_then(remove_all_txs_from_submission_material_in_state)
        .and_then(add_latest_algo_submission_material_to_db_and_return_state)
        .and_then(|state| {
            initialize_algo_chain_db_keys(
                &state.algo_db_utils,
                &state.get_algo_submission_material()?.block.hash()?,
                canon_to_tip_length,
            )?;
            Ok(state)
        })
}

/// Debug Reset ALGO Chain
///
/// This function will reset the ALGO chain held in the encrypted database. It first deletes the
/// entire chain, working backwards from the current latest block. It then deletes the relevant
/// database keys pertaining to the head, tail, anchor and canon block hashes of the chain.
/// Finally, it uses the passed in submission material to re-initialize these values from the
/// included block.
///
/// ### Beware: The block used to reset the chain must be trusted. Use this function only if you
/// know exactly what you are doing and why.
#[named]
pub fn debug_reset_algo_chain<D: DatabaseInterface>(
    db: &D,
    block_json_string: &str,
    canon_to_tip_length: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("Debug resetting ALGO chain...");
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json_string, &canon_to_tip_length, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
        .and_then(|_| parse_algo_submission_material_and_put_in_state(block_json_string, AlgoState::init(db)))
        .and_then(|state| reset_algo_chain_and_return_state(state, canon_to_tip_length))
        .and_then(end_algo_db_transaction_and_return_state)
        .map(|_| {
            json!({
                "algo-chain-reset-success":true,
                "number_of_confirmations": if canon_to_tip_length == 0 {
                    ZERO_CONFS_WARNING.to_string()
                } else {
                    canon_to_tip_length.to_string()
                },
            })
            .to_string()
        })
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::test_utils::get_sample_contiguous_submission_material;

    #[test]
    fn should_delete_all_algo_submission_material() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_sample_contiguous_submission_material();
        let block_hashes = submission_materials
            .iter()
            .map(|material| material.block.hash().unwrap())
            .collect::<Vec<AlgorandHash>>();
        let latest_hash = block_hashes[block_hashes.len() - 1];
        submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(material).unwrap());
        db_utils.put_latest_block_hash_in_db(&latest_hash).unwrap();
        block_hashes
            .iter()
            .for_each(|hash| assert!(db_utils.get_submission_material(hash).is_ok()));
        delete_all_algo_submision_material(&db_utils).unwrap();
        block_hashes.iter().enumerate().for_each(|(i, hash)| {
            let result = db_utils.get_submission_material(hash);
            if result.is_ok() {
                let err_msg = format!(
                    "Sample ALGO submission_material #{} still exists in DB under hash: 0x{}",
                    i, hash,
                );
                panic!("{}", err_msg);
            }
        });
    }
}
