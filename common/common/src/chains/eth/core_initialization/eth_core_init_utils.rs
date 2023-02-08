use crate::{
    chains::eth::{eth_chain_id::EthChainId, eth_database_utils::EthDbUtilsExt, EthState},
    traits::DatabaseInterface,
    types::Result,
};

fn put_tail_block_hash_in_db_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Putting ETH tail block has in db...");
    let block_hash = state.get_eth_submission_material()?.get_block_hash()?;
    if is_for_eth {
        state.eth_db_utils.put_eth_tail_block_hash_in_db(&block_hash)?;
    } else {
        state.evm_db_utils.put_eth_tail_block_hash_in_db(&block_hash)?;
    };
    Ok(state)
}

pub fn put_eth_tail_block_hash_in_db_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    put_tail_block_hash_in_db_and_return_state(true, state)
}

pub fn put_evm_tail_block_hash_in_db_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    put_tail_block_hash_in_db_and_return_state(false, state)
}

fn set_hash_from_block_in_state<'a, D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<'a, D>,
    hash_type: &str,
) -> Result<EthState<'a, D>> {
    let hash = state.get_eth_submission_material()?.get_block_hash()?;
    let block_type = if is_for_eth { "ETH" } else { "EVM" };
    match hash_type {
        "canon" => {
            info!("✔ Initializating {} canon block hash...", block_type);
            if is_for_eth {
                state.eth_db_utils.put_eth_canon_block_hash_in_db(&hash)
            } else {
                state.evm_db_utils.put_eth_canon_block_hash_in_db(&hash)
            }
        },
        "latest" => {
            info!("✔ Initializating {} latest block hash...", block_type);
            if is_for_eth {
                state.eth_db_utils.put_eth_latest_block_hash_in_db(&hash)
            } else {
                state.evm_db_utils.put_eth_latest_block_hash_in_db(&hash)
            }
        },
        "anchor" => {
            info!("✔ Initializating {} anchor block hash...", block_type);
            if is_for_eth {
                state.eth_db_utils.put_eth_anchor_block_hash_in_db(&hash)
            } else {
                state.evm_db_utils.put_eth_anchor_block_hash_in_db(&hash)
            }
        },
        _ => Err("✘ Hash type not recognized!".into()),
    }?;
    Ok(state)
}

pub fn set_eth_latest_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(true, state, "latest")
}

pub fn set_eth_anchor_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(true, state, "anchor")
}

pub fn set_eth_canon_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(true, state, "canon")
}

pub fn set_evm_latest_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(false, state, "latest")
}

pub fn set_evm_anchor_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(false, state, "anchor")
}

pub fn set_evm_canon_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(false, state, "canon")
}

fn put_canon_to_tip_length_in_db_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    canon_to_tip_length: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    if is_for_eth {
        info!(
            "✔ Putting {} canon-to-tip lenght of {} into db...",
            if is_for_eth { "ETH" } else { "EVM" },
            canon_to_tip_length
        );
        state
            .eth_db_utils
            .put_eth_canon_to_tip_length_in_db(canon_to_tip_length)?;
    } else {
        state
            .evm_db_utils
            .put_eth_canon_to_tip_length_in_db(canon_to_tip_length)?;
    };
    Ok(state)
}

pub fn put_eth_canon_to_tip_length_in_db_and_return_state<D: DatabaseInterface>(
    canon_to_tip_length: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    put_canon_to_tip_length_in_db_and_return_state(true, canon_to_tip_length, state)
}

pub fn put_evm_canon_to_tip_length_in_db_and_return_state<D: DatabaseInterface>(
    canon_to_tip_length: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    put_canon_to_tip_length_in_db_and_return_state(false, canon_to_tip_length, state)
}

fn put_chain_id_in_db_and_return_state<'a, D: DatabaseInterface>(
    is_for_eth: bool,
    chain_id: &EthChainId,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    info!(
        "✔ Putting {} chain ID of {} in db...",
        if is_for_eth { "ETH" } else { "EVM" },
        chain_id
    );
    if is_for_eth {
        state.eth_db_utils.put_eth_chain_id_in_db(chain_id)?;
    } else {
        state.evm_db_utils.put_eth_chain_id_in_db(chain_id)?;
    };
    Ok(state)
}

pub fn put_eth_chain_id_in_db_and_return_state<'a, D: DatabaseInterface>(
    chain_id: &EthChainId,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    put_chain_id_in_db_and_return_state(true, chain_id, state)
}

pub fn put_evm_chain_id_in_db_and_return_state<'a, D: DatabaseInterface>(
    chain_id: &EthChainId,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    put_chain_id_in_db_and_return_state(false, chain_id, state)
}

fn put_gas_price_in_db_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    gas_price: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!(
        "✔ Putting {} gas price of {} in db...",
        if is_for_eth { "ETH" } else { "EVM" },
        gas_price
    );
    if is_for_eth {
        state.eth_db_utils.put_eth_gas_price_in_db(gas_price)?;
    } else {
        state.evm_db_utils.put_eth_gas_price_in_db(gas_price)?;
    };
    Ok(state)
}

pub fn put_eth_gas_price_in_db_and_return_state<D: DatabaseInterface>(
    gas_price: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    put_gas_price_in_db_and_return_state(true, gas_price, state)
}

pub fn put_evm_gas_price_in_db_and_return_state<D: DatabaseInterface>(
    gas_price: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    put_gas_price_in_db_and_return_state(false, gas_price, state)
}

fn put_account_nonce_in_db_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    nonce: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!(
        "✔ Putting {} account nonce of {} in db...",
        nonce,
        if is_for_eth { "ETH" } else { "EVM" }
    );
    if is_for_eth {
        state.eth_db_utils.put_eth_account_nonce_in_db(nonce)?;
    } else {
        state.evm_db_utils.put_eth_account_nonce_in_db(nonce)?;
    };
    Ok(state)
}

pub fn put_eth_account_nonce_in_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
    nonce: u64,
) -> Result<EthState<D>> {
    put_account_nonce_in_db_and_return_state(true, nonce, state)
}

pub fn put_evm_account_nonce_in_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
    nonce: u64,
) -> Result<EthState<D>> {
    put_account_nonce_in_db_and_return_state(false, nonce, state)
}

pub fn remove_receipts_from_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    // ∵ there shouldn't be relevant txs!
    info!("✔ Removing receipts from ETH block in state...");
    let submission_material_with_no_receipts = state.get_eth_submission_material()?.remove_receipts();
    state.update_eth_submission_material(submission_material_with_no_receipts)
}

fn add_block_to_db_and_return_state<D: DatabaseInterface>(is_for_eth: bool, state: EthState<D>) -> Result<EthState<D>> {
    info!(
        "✔ Adding {} block and receipts to db...",
        if is_for_eth { "ETH" } else { "EVM" }
    );
    let submission_material = state.get_eth_submission_material()?;
    if is_for_eth {
        state
            .eth_db_utils
            .put_eth_submission_material_in_db(submission_material)?;
    } else {
        state
            .evm_db_utils
            .put_eth_submission_material_in_db(submission_material)?;
    };
    Ok(state)
}

pub fn add_eth_block_to_db_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    add_block_to_db_and_return_state(true, state)
}

pub fn add_evm_block_to_db_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    add_block_to_db_and_return_state(false, state)
}

fn put_any_sender_nonce_in_db_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    const ANY_SENDER_START_NONCE: u64 = 0;
    info!(
        "✔ Putting {} AnySender nonce of {} in db...",
        if is_for_eth { "ETH" } else { "EVM" },
        ANY_SENDER_START_NONCE
    );
    if is_for_eth {
        state.eth_db_utils.put_any_sender_nonce_in_db(ANY_SENDER_START_NONCE)?;
    } else {
        state.evm_db_utils.put_any_sender_nonce_in_db(ANY_SENDER_START_NONCE)?;
    };
    Ok(state)
}

pub fn put_eth_any_sender_nonce_in_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    put_any_sender_nonce_in_db_and_return_state(true, state)
}

pub fn put_evm_any_sender_nonce_in_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    put_any_sender_nonce_in_db_and_return_state(false, state)
}
