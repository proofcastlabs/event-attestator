use ethereum_types::H256 as EthHash;
use function_name::named;
use serde_json::json;

use crate::{
    chains::eth::{
        core_initialization::eth_core_init_utils::{
            add_eth_block_to_db_and_return_state,
            add_evm_block_to_db_and_return_state,
            put_eth_canon_to_tip_length_in_db_and_return_state,
            put_eth_tail_block_hash_in_db_and_return_state,
            put_evm_canon_to_tip_length_in_db_and_return_state,
            put_evm_tail_block_hash_in_db_and_return_state,
            remove_receipts_from_block_in_state,
            set_eth_anchor_block_hash_and_return_state,
            set_eth_canon_block_hash_and_return_state,
            set_eth_latest_block_hash_and_return_state,
            set_evm_anchor_block_hash_and_return_state,
            set_evm_canon_block_hash_and_return_state,
            set_evm_latest_block_hash_and_return_state,
        },
        eth_database_transactions::end_eth_db_transaction_and_return_state,
        eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
        eth_state::EthState,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        validate_block_in_state::validate_block_in_state,
    },
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    traits::DatabaseInterface,
    types::Result,
};

fn delete_all_eth_blocks<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<()> {
    fn recursively_delete_all_eth_blocks<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
        db_utils: &E,
        maybe_block_hash: Option<EthHash>,
    ) -> Result<()> {
        match maybe_block_hash {
            None => {
                info!("✔ Deleting all ETH blocks from db, starting with the latest block...");
                recursively_delete_all_eth_blocks(db_utils, Some(db_utils.get_special_eth_hash_from_db("latest")?))
            },
            Some(ref hash) => match db_utils.get_submission_material_from_db(hash) {
                Ok(submission_material) => {
                    info!("✔ Deleting block {}...", submission_material.get_block_number()?);
                    db_utils.delete_block_by_block_hash(&submission_material).and_then(|_| {
                        recursively_delete_all_eth_blocks(db_utils, Some(submission_material.get_parent_hash()?))
                    })
                },
                Err(_) => {
                    info!("✔ All ETH blocks deleted!");
                    Ok(())
                },
            },
        }
    }
    recursively_delete_all_eth_blocks(db_utils, None)
}

fn delete_all_relevant_db_keys<D: DatabaseInterface>(db: &D, is_for_eth: bool) -> Result<()> {
    let evm_db_utils = EvmDbUtils::new(db);
    let eth_db_utils = EthDbUtils::new(db);
    if is_for_eth {
        vec![
            eth_db_utils.get_eth_linker_hash_key(),
            eth_db_utils.get_eth_canon_block_hash_key(),
            eth_db_utils.get_eth_tail_block_hash_key(),
            eth_db_utils.get_eth_ptoken_genesis_hash_key(),
            eth_db_utils.get_eth_anchor_block_hash_key(),
            eth_db_utils.get_eth_latest_block_hash_key(),
            eth_db_utils.get_eth_canon_to_tip_length_key(),
        ]
    } else {
        vec![
            evm_db_utils.get_evm_linker_hash_key(),
            evm_db_utils.get_evm_tail_block_hash_key(),
            evm_db_utils.get_evm_ptoken_genesis_hash_key(),
            evm_db_utils.get_evm_anchor_block_hash_key(),
            evm_db_utils.get_evm_latest_block_hash_key(),
            evm_db_utils.get_evm_canon_block_hash_key(),
            evm_db_utils.get_evm_canon_to_tip_length_key(),
        ]
    }
    .iter()
    .map(|key| db.delete(key.to_vec()))
    .collect::<Result<Vec<()>>>()
    .and(Ok(()))
}

fn delete_all_blocks_and_db_keys_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
    is_for_eth: bool,
) -> Result<EthState<D>> {
    if is_for_eth {
        delete_all_eth_blocks(&state.eth_db_utils)
            .and_then(|_| delete_all_relevant_db_keys(state.db, is_for_eth)) // TODO make a util for this!
            .and(Ok(state))
    } else {
        delete_all_eth_blocks(&state.evm_db_utils)
            .and_then(|_| delete_all_relevant_db_keys(state.db, is_for_eth)) // TODO Ibid.
            .and(Ok(state))
    }
}

pub fn reset_eth_chain<D: DatabaseInterface>(
    state: EthState<D>,
    canon_to_tip_length: u64,
    is_for_eth: bool,
) -> Result<EthState<D>> {
    info!("Resetting ETH chain...");
    delete_all_blocks_and_db_keys_and_return_state(state, is_for_eth)
        .and_then(remove_receipts_from_block_in_state)
        .and_then(|state| {
            if is_for_eth {
                add_eth_block_to_db_and_return_state(state)
            } else {
                add_evm_block_to_db_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_canon_to_tip_length_in_db_and_return_state(canon_to_tip_length, state)
            } else {
                put_evm_canon_to_tip_length_in_db_and_return_state(canon_to_tip_length, state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_anchor_block_hash_and_return_state(state)
            } else {
                set_evm_anchor_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_latest_block_hash_and_return_state(state)
            } else {
                set_evm_latest_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_canon_block_hash_and_return_state(state)
            } else {
                set_evm_canon_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_tail_block_hash_in_db_and_return_state(state)
            } else {
                put_evm_tail_block_hash_in_db_and_return_state(state)
            }
        })
}

#[named]
fn debug_reset_chain<D: DatabaseInterface>(
    db: &D,
    submission_material_json: &str,
    canon_to_tip_length: u64,
    is_for_eth: bool,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("Debug resetting ETH chain...");
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| {
            get_debug_command_hash!(
                function_name!(),
                &submission_material_json,
                &canon_to_tip_length,
                &is_for_eth,
                core_type
            )()
        })
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| parse_eth_submission_material_and_put_in_state(submission_material_json, EthState::init(db)))
        .and_then(validate_block_in_state)
        .and_then(|state| reset_eth_chain(state, canon_to_tip_length, is_for_eth))
        .and_then(end_eth_db_transaction_and_return_state)
        .map(|_| {
            let json = if is_for_eth {
                json!({"eth-chain-reset-success":true})
            } else {
                json!({"evm-chain-reset-success":true})
            };
            json.to_string()
        })
}

/// Debug Reset ETH Chain
///
/// This function will reset the ETH chain held in the encrypted database. It first deletes the
/// entire chain, working backwards from the current latest block. It then deletes the relevant
/// database keys pertaining to the head, tail, anchor and canon block hashes of the chain.
/// Finally, it uses the passed in submission material to re-initialize these values from the
/// included block.
///
/// ### Beware: The block used to reset the chain must be trusted. Use this function only if you
/// know exactly what you are doing and why.
pub fn debug_reset_eth_chain<D: DatabaseInterface>(
    db: &D,
    submission_material_json: &str,
    canon_to_tip_length: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("Debug resetting ETH chain...");
    debug_reset_chain(
        db,
        submission_material_json,
        canon_to_tip_length,
        true,
        core_type,
        signature,
    )
}

/// Debug Reset EVM Chain
///
/// This function will reset the EVM Chain held in the encrypted database. It first deletes the
/// entire chain, working backwards from the current latest block. It then deletes the relevant
/// database keys pertaining to the head, tail, anchor and canon block hashes of the chain.
/// Finally, it uses the passed in submission material to re-initialize these values from the
/// included block.
///
/// ### Beware: The block used to reset the chain must be trusted. Use this function only if you
/// know exactly what you are doing and why.
pub fn debug_reset_evm_chain<D: DatabaseInterface>(
    db: &D,
    submission_material_json: &str,
    canon_to_tip_length: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("Debug resetting EVM Chain...");
    debug_reset_chain(
        db,
        submission_material_json,
        canon_to_tip_length,
        false,
        core_type,
        signature,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::eth::eth_test_utils::get_sequential_eth_blocks_and_receipts, test_utils::get_test_database};

    #[test]
    fn should_recursively_delete_all_eth_blocks() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let block_hashes = blocks
            .clone()
            .iter()
            .map(|block| block.hash.unwrap())
            .collect::<Vec<EthHash>>();
        let latest_hash = block_hashes[block_hashes.len() - 1];
        blocks
            .iter()
            .for_each(|material| db_utils.put_eth_submission_material_in_db(material).unwrap());
        db_utils.put_eth_latest_block_hash_in_db(&latest_hash).unwrap();
        block_hashes
            .iter()
            .for_each(|hash| assert!(db_utils.get_submission_material_from_db(&hash).is_ok()));
        delete_all_eth_blocks(&db_utils).unwrap();
        block_hashes.iter().enumerate().for_each(|(i, hash)| {
            let result = db_utils.get_submission_material_from_db(&hash);
            if result.is_ok() {
                let err_msg = format!(
                    "Sample ETH block #{} still exists in DB under hash: 0x{}",
                    i,
                    hex::encode(hash.as_bytes())
                );
                assert!(false, "{}", err_msg);
            }
        });
    }

    #[test]
    fn should_recursively_delete_all_evm_blocks() {
        let db = get_test_database();
        let db_utils = EvmDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let block_hashes = blocks
            .clone()
            .iter()
            .map(|block| block.hash.unwrap())
            .collect::<Vec<EthHash>>();
        let latest_hash = block_hashes[block_hashes.len() - 1];
        blocks
            .iter()
            .for_each(|material| db_utils.put_eth_submission_material_in_db(material).unwrap());
        db_utils.put_eth_latest_block_hash_in_db(&latest_hash).unwrap();
        block_hashes
            .iter()
            .for_each(|hash| assert!(db_utils.get_submission_material_from_db(&hash).is_ok()));
        delete_all_eth_blocks(&db_utils).unwrap();
        block_hashes.iter().enumerate().for_each(|(i, hash)| {
            let result = db_utils.get_submission_material_from_db(&hash);
            if result.is_ok() {
                let err_msg = format!(
                    "Sample EVM block #{} still exists in DB under hash: 0x{}",
                    i,
                    hex::encode(hash.as_bytes())
                );
                assert!(false, "{}", err_msg);
            }
        });
    }
}
