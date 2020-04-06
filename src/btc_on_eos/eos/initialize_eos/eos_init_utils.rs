use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ProcessedTxIds,
        eos_crypto::eos_private_key::EosPrivateKey,
        parse_submission_material::parse_producer_schedule_from_json_string,
        eos_database_utils::{
            put_eos_schedule_in_db,
            put_eos_chain_id_in_db,
            put_eos_private_key_in_db,
            put_eos_account_name_in_db,
            put_eos_token_symbol_in_db,
            put_processed_tx_ids_in_db,
            put_eos_account_nonce_in_db,
        },
    },
};

pub fn put_eos_schedule_in_db_and_return_state<D>(
    schedule_json: &String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    parse_producer_schedule_from_json_string(schedule_json)
        .and_then(|schedule| put_eos_schedule_in_db(&state.db, &schedule))
        .and(Ok(state))
}

pub fn generated_eos_key_save_in_db_and_return_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_private_key_in_db(
        &state.db,
        &EosPrivateKey::generate_random()?,
    )
        .and(Ok(state))
}

pub fn get_eos_init_output<D>(
    _state: EosState<D>
) -> Result<String>
    where D: DatabaseInterface
{
    Ok("{eos_core_initialized:true}".to_string())
}

pub fn put_eos_account_name_in_db_and_return_state<D>(
    account_name: String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_account_name_in_db(&state.db, &account_name)
        .and(Ok(state))
}

pub fn put_eos_account_nonce_in_db_and_return_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_account_nonce_in_db(&state.db, 0)
        .and(Ok(state))
}

pub fn put_eos_token_symbol_in_db_and_return_state<D>(
    token_symbol: String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_token_symbol_in_db(&state.db, &token_symbol)
        .and(Ok(state))
}

pub fn put_empty_processed_tx_ids_in_db_and_return_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_processed_tx_ids_in_db(
        &state.db,
        &ProcessedTxIds::init()
    )
        .and(Ok(state))
}

pub fn put_eos_chain_id_in_db_and_return_state<D>(
    chain_id: String,
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_chain_id_in_db(
        &state.db,
        &chain_id,
    )
        .and(Ok(state))
}
