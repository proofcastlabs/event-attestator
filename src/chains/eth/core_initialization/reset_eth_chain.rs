use ethereum_types::H256 as EthHash;
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
        eth_constants::{
            ETH_ANCHOR_BLOCK_HASH_KEY,
            ETH_CANON_BLOCK_HASH_KEY,
            ETH_CANON_TO_TIP_LENGTH_KEY,
            ETH_LATEST_BLOCK_HASH_KEY,
            ETH_LINKER_HASH_KEY,
            ETH_TAIL_BLOCK_HASH_KEY,
            PTOKEN_GENESIS_HASH_KEY,
        },
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        evm_constants::{
            EVM_ANCHOR_BLOCK_HASH_KEY,
            EVM_CANON_BLOCK_HASH_KEY,
            EVM_CANON_TO_TIP_LENGTH_KEY,
            EVM_LATEST_BLOCK_HASH_KEY,
            EVM_LINKER_HASH_KEY,
            EVM_PTOKEN_GENESIS_HASH_KEY,
            EVM_TAIL_BLOCK_HASH_KEY,
        },
        validate_block_in_state::validate_block_in_state,
    },
    check_debug_mode::check_debug_mode,
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
                recursively_delete_all_eth_blocks(
                    db_utils,
                    Some(db_utils.get_eth_latest_block_from_db()?.get_parent_hash()?),
                )
            },
            Some(ref hash) => match db_utils.get_submission_material_from_db(hash) {
                Ok(submission_material) => {
                    recursively_delete_all_eth_blocks(db_utils, Some(submission_material.get_parent_hash()?))
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
    if is_for_eth {
        vec![
            *ETH_LINKER_HASH_KEY,
            *ETH_CANON_BLOCK_HASH_KEY,
            *ETH_TAIL_BLOCK_HASH_KEY,
            *PTOKEN_GENESIS_HASH_KEY,
            *ETH_ANCHOR_BLOCK_HASH_KEY,
            *ETH_LATEST_BLOCK_HASH_KEY,
            *ETH_CANON_TO_TIP_LENGTH_KEY,
        ]
    } else {
        vec![
            *EVM_LINKER_HASH_KEY,
            *EVM_TAIL_BLOCK_HASH_KEY,
            *EVM_PTOKEN_GENESIS_HASH_KEY,
            *EVM_ANCHOR_BLOCK_HASH_KEY,
            *EVM_LATEST_BLOCK_HASH_KEY,
            *EVM_CANON_BLOCK_HASH_KEY,
            *EVM_CANON_TO_TIP_LENGTH_KEY,
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

fn debug_reset_chain<D: DatabaseInterface>(
    db: D,
    submission_material_json: &str,
    canon_to_tip_length: u64,
    is_for_eth: bool,
) -> Result<String> {
    info!("Debug resetting ETH chain...");
    check_debug_mode()
        .and_then(|_| parse_eth_submission_material_and_put_in_state(submission_material_json, EthState::init(&db)))
        .and_then(validate_block_in_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(|state| delete_all_blocks_and_db_keys_and_return_state(state, is_for_eth))
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
    db: D,
    submission_material_json: &str,
    canon_to_tip_length: u64,
) -> Result<String> {
    info!("Debug resetting ETH chain...");
    debug_reset_chain(db, submission_material_json, canon_to_tip_length, true)
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
    db: D,
    submission_material_json: &str,
    canon_to_tip_length: u64,
) -> Result<String> {
    info!("Debug resetting EVM Chain...");
    debug_reset_chain(db, submission_material_json, canon_to_tip_length, false)
}
