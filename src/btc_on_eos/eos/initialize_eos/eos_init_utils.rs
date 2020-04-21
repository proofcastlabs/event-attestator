use crate::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    btc_on_eos::{
        utils::convert_hex_to_checksum256,
        eos::{
            eos_state::EosState,
            eos_merkle_utils::IncreMerkle,
            eos_crypto::eos_private_key::EosPrivateKey,
            validate_signature::check_block_signature_is_valid,
            parse_eos_schedule::parse_schedule_string_to_schedule,
            parse_submission_material::parse_eos_block_header_from_json,
            eos_types::{
                Checksum256s,
                ProcessedTxIds,
                EosKnownSchedules,
                EosBlockHeaderJson,
            },
            eos_database_utils::{
                put_incremerkle_in_db,
                put_eos_schedule_in_db,
                put_eos_chain_id_in_db,
                get_incremerkle_from_db,
                get_eos_schedule_from_db,
                put_eos_private_key_in_db,
                put_eos_account_name_in_db,
                put_eos_token_symbol_in_db,
                put_processed_tx_ids_in_db,
                put_eos_account_nonce_in_db,
                put_eos_known_schedules_in_db,
                put_eos_last_seen_block_id_in_db,
                put_eos_last_seen_block_num_in_db,
                get_eos_last_seen_block_num_from_db,
            },
        },
    },
};

pub fn test_block_validation_and_return_state<D>(
    block_json: &String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Checking block validation passes...");
    let header_json: EosBlockHeaderJson = match serde_json::from_str(
        block_json
    ) {
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }?;
    check_block_signature_is_valid(
        &get_incremerkle_from_db(&state.db)?
            .get_root()
            .to_bytes()
            .to_vec(),
        &header_json.producer_signature,
        &parse_eos_block_header_from_json(&header_json)?,
        &get_eos_schedule_from_db(&state.db, header_json.schedule_version)?
    )
        .and(Ok(state))
}

pub fn generate_and_put_incremerkle_in_db_and_return_state<D>(
    blockroot_merkle_json: &String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing blockroot merkle json string...");
    let blockroot_merkle: Vec<String> = match serde_json::from_str(
        blockroot_merkle_json
    ) {
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }?;
    info!("✔ Generating and putting incremerkle in db...");
    put_incremerkle_in_db(
        &state.db,
        &IncreMerkle::new( // FIXME Do we need to add block id to this?
            get_eos_last_seen_block_num_from_db(&state.db)?,
            blockroot_merkle
                .iter()
                .map(convert_hex_to_checksum256)
                .collect::<Result<Checksum256s>>()?
        ),
    )
        .and(Ok(state))
}

pub fn put_eos_latest_block_info_in_db_and_return_state<D>(
    block_json: &String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    let json: EosBlockHeaderJson = match serde_json::from_str(block_json) {
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }?;
    let id = json.block_id.clone();
    let num = json.block_num.clone();
    info!("✔ Putting latest block number '{}' & ID '{}' into db...", num, id);
    put_eos_last_seen_block_num_in_db(&state.db, num)
        .and_then(|_|
            put_eos_last_seen_block_id_in_db(
                &state.db,
                &convert_hex_to_checksum256(id)?
            )
        )
        .and(Ok(state))
}

pub fn put_eos_known_schedule_in_db_and_return_state<D>(
    schedule_json: &String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Putting EOS known schedule into db...");
    parse_schedule_string_to_schedule(schedule_json)
        .map(|sched| EosKnownSchedules::new(sched.version))
        .and_then(|sched| put_eos_known_schedules_in_db(&state.db, &sched))
        .and(Ok(state))
}

pub fn put_eos_schedule_in_db_and_return_state<D>(
    schedule_json: &String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Putting EOS schedule into db...");
    parse_schedule_string_to_schedule(schedule_json)
        .and_then(|schedule| put_eos_schedule_in_db(&state.db, &schedule))
        .and(Ok(state))
}

pub fn generated_eos_key_save_in_db_and_return_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Generating EOS private key & putting into db...");
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
    info!("✔ Putting EOS account name '{}' into db...", account_name);
    put_eos_account_name_in_db(&state.db, &account_name)
        .and(Ok(state))
}

pub fn put_eos_account_nonce_in_db_and_return_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Putting EOS account nonce in db...");
    put_eos_account_nonce_in_db(&state.db, 0)
        .and(Ok(state))
}

pub fn put_eos_token_symbol_in_db_and_return_state<D>(
    token_symbol: String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Putting EOS token symbol '{}' into db...", token_symbol);
    put_eos_token_symbol_in_db(&state.db, &token_symbol)
        .and(Ok(state))
}

pub fn put_empty_processed_tx_ids_in_db_and_return_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Initializing EOS processed tx ids & putting into db...");
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
    info!("✔ Putting EOS chain ID '{}' into db...", chain_id);
    put_eos_chain_id_in_db(
        &state.db,
        &chain_id,
    )
        .and(Ok(state))
}
