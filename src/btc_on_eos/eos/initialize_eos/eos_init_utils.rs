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
            parse_eos_schedule::{
                EosProducerScheduleJson,
                convert_schedule_json_to_schedule_v2,
            },
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosInitJson {
    pub block: EosBlockHeaderJson,
    pub blockroot_merkle: Vec<String>,
    pub active_schedule: EosProducerScheduleJson,
}

impl EosInitJson {
    pub fn from_json_string(json_string: &String) -> Result<Self> {
        match serde_json::from_str(&json_string) {
            Ok(result) => Ok(result),
            Err(e) => Err(AppError::Custom(e.to_string()))
        }
    }
}

pub fn parse_eos_init_json_from_string( // TODO use impl instead!
    eos_init_json: String,
) -> Result<EosInitJson> {
    match serde_json::from_str(&eos_init_json) {
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }
}

pub fn test_block_validation_and_return_state<D>(
    block_json: &EosBlockHeaderJson,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Checking block validation passes...");
    check_block_signature_is_valid(
        &get_incremerkle_from_db(&state.db)?
            .get_root()
            .to_bytes()
            .to_vec(),
        &block_json.producer_signature,
        &parse_eos_block_header_from_json(&block_json)?,
        &get_eos_schedule_from_db(&state.db, block_json.schedule_version)?
    )
        .and(Ok(state))
}

pub fn generate_and_put_incremerkle_in_db<D>(
    db: &D,
    blockroot_merkle: &Vec<String>,
) -> Result<()>
    where D: DatabaseInterface
{
    info!("✔ Generating and putting incremerkle in db...");
    put_incremerkle_in_db(
        db,
        &IncreMerkle::new(
            get_eos_last_seen_block_num_from_db(db)? - 1,
            blockroot_merkle
                .iter()
                .map(convert_hex_to_checksum256)
                .collect::<Result<Checksum256s>>()?
        ),
    )
}

pub fn generate_and_put_incremerkle_in_db_and_return_state<D>(
    blockroot_merkle: &Vec<String>,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    generate_and_put_incremerkle_in_db(&state.db, &blockroot_merkle)
        .and(Ok(state))
}

pub fn put_eos_latest_block_info_in_db<D>(
    db: &D,
    block_json: &EosBlockHeaderJson,
) -> Result<()>
    where D: DatabaseInterface
{
    info!(
        "✔ Putting latest block number '{}' & ID '{}' into db...",
        &block_json.block_num,
        &block_json.block_id
    );
    put_eos_last_seen_block_num_in_db(db, block_json.block_num)
        .and_then(|_|
            put_eos_last_seen_block_id_in_db(
                db,
                &convert_hex_to_checksum256(block_json.block_id.clone())?
            )
        )
}

pub fn put_eos_latest_block_info_in_db_and_return_state<D>(
    block_json: &EosBlockHeaderJson,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_latest_block_info_in_db(&state.db, block_json)
        .and(Ok(state))
}

pub fn put_eos_known_schedule_in_db_and_return_state<D>(
    schedule_json: &EosProducerScheduleJson,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Putting EOS known schedule into db...");
    convert_schedule_json_to_schedule_v2(schedule_json)
        .map(|sched| EosKnownSchedules::new(sched.version))
        .and_then(|sched| put_eos_known_schedules_in_db(&state.db, &sched))
        .and(Ok(state))
}

pub fn put_eos_schedule_in_db_and_return_state<D>(
    schedule_json: &EosProducerScheduleJson,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Putting EOS schedule into db...");
    convert_schedule_json_to_schedule_v2(schedule_json)
        .and_then(|schedule| put_eos_schedule_in_db(&state.db, &schedule))
        .and(Ok(state))
}

pub fn generated_eos_key_save_in_db_and_return_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Generating EOS private key & putting into db...");
    EosPrivateKey::generate_random()?
        .write_to_db(&state.db)
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

#[cfg(test)]
mod tests {
    use super::*;
    use eos_primitives::Checksum256;
    use crate::btc_on_eos::{
        utils::convert_hex_to_checksum256,
        eos::{
            eos_merkle_utils::IncreMerkle,
            parse_eos_schedule::convert_schedule_json_to_schedule_v2,
            parse_submission_material::parse_eos_block_header_from_json,
            eos_test_utils::{
                get_init_json_n,
                NUM_INIT_SAMPLES,
            },
        },
    };

    fn validate_init_block(init_json: &EosInitJson) {
        let schedule = convert_schedule_json_to_schedule_v2(
            &init_json.active_schedule
        ).unwrap();
        let block_header = parse_eos_block_header_from_json(
            &init_json.block
        ).unwrap();
        let blockroot_merkle = init_json
            .blockroot_merkle
            .iter()
            .map(|hex| convert_hex_to_checksum256(hex))
            .collect::<Result<Vec<Checksum256>>>()
            .unwrap();
        let producer_signature = init_json
            .block
            .producer_signature
            .clone();
        let incremerkle = IncreMerkle::new(
            (block_header.block_num() - 1).into(),
            blockroot_merkle,
        );
        let block_mroot = incremerkle
            .get_root()
            .to_bytes()
            .to_vec();
        if let Err(_) = check_block_signature_is_valid(
            &block_mroot,
            &producer_signature,
            &block_header,
            &schedule,
        ) {
            panic!("Could not validate init block!");
        }
    }

    #[test]
    fn should_validate_jungle_3_init_blocks() {
        vec![0; NUM_INIT_SAMPLES]
            .iter()
            .enumerate()
            .map(|(i, _)| get_init_json_n(i + 1))
            .collect::<Result<Vec<EosInitJson>>>()
            .unwrap()
            .iter()
            .map(|init_json| validate_init_block(init_json))
            .for_each(drop);
    }
}
