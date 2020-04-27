use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::{
        eos_state::EosState,
        eos_database_utils::{
            end_eos_db_transaction,
            start_eos_db_transaction,
        },
        initialize_eos::{
            is_eos_core_initialized::is_eos_core_initialized,
            eos_init_utils::{
                get_eos_init_output,
                parse_eos_init_json_from_string,
                test_block_validation_and_return_state,
                put_eos_schedule_in_db_and_return_state,
                put_eos_chain_id_in_db_and_return_state,
                put_eos_token_symbol_in_db_and_return_state,
                put_eos_account_name_in_db_and_return_state,
                put_eos_account_nonce_in_db_and_return_state,
                put_eos_known_schedule_in_db_and_return_state,
                generated_eos_key_save_in_db_and_return_state,
                put_eos_latest_block_info_in_db_and_return_state,
                put_empty_processed_tx_ids_in_db_and_return_state,
                generate_and_put_incremerkle_in_db_and_return_state,
            },
        },
    },
};

pub fn maybe_initialize_eos_core<D>(
    db: D,
    chain_id: String,
    account_name: String,
    token_symbol: String,
    eos_init_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Maybe initializing EOS core...");
    let init_json = parse_eos_init_json_from_string(eos_init_json)?;
    match is_eos_core_initialized(&db) {
        true => {
            info!("✔ EOS core already initialized!");
            Ok("{eos_core_initialized:true}".to_string())
        }
        false => {
            info!("✔ Initializing core for EOS...");
            start_eos_db_transaction(EosState::init(db))
                .and_then(|state| {
                    put_empty_processed_tx_ids_in_db_and_return_state(state)
                })
                .and_then(|state|
                    put_eos_chain_id_in_db_and_return_state(
                        chain_id,
                        state,
                    )
                )
                .and_then(|state|
                    put_eos_account_name_in_db_and_return_state(
                        account_name,
                        state,
                    )
                )
                .and_then(|state|
                    put_eos_token_symbol_in_db_and_return_state(
                        token_symbol,
                        state,
                    )
                )
                .and_then(|state|
                    put_eos_known_schedule_in_db_and_return_state(
                        &init_json.active_schedule,
                        state,
                    )
                )
                .and_then(|state|
                    put_eos_schedule_in_db_and_return_state(
                        &init_json.active_schedule,
                        state,
                    )
                )
                .and_then(|state|
                    put_eos_latest_block_info_in_db_and_return_state(
                        &init_json.block,
                        state,
                    )
                )
                .and_then(|state|
                    generate_and_put_incremerkle_in_db_and_return_state(
                        &init_json.blockroot_merkle,
                        state,
                    )
                )
                .and_then(|state|
                    test_block_validation_and_return_state(
                        &init_json.block,
                        state,
                    )
                )
                .and_then(generated_eos_key_save_in_db_and_return_state)
                .and_then(put_eos_account_nonce_in_db_and_return_state)
                .and_then(end_eos_db_transaction)
                .and_then(get_eos_init_output)
        }
    }
}
