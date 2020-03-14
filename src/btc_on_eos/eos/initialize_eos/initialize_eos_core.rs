use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    utils::convert_hex_to_checksum256,
    eos::{
        eos_state::EosState,
        eos_types::ProcessedTxIds,
        initialize_eos::is_eos_core_initialized::is_eos_core_initialized,
        eos_database_utils::{
            put_eos_chain_id_in_db,
            end_eos_db_transaction,
            start_eos_db_transaction,
            put_eos_account_name_in_db,
            put_eos_token_ticker_in_db,
            put_processed_tx_ids_in_db,
            get_processed_tx_ids_from_db,
        },
    },
};

// TODO move to init_utils
fn get_eos_init_output<D>(
    _state: EosState<D>
) -> Result<String>
    where D: DatabaseInterface
{
    Ok("{eos_core_initialized:true}".to_string())
}

fn put_eos_account_name_in_db_and_return_state<D>(
    account_name: String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_account_name_in_db(&state.db, &account_name)
        .map(|_| state)
}

fn put_eos_token_ticker_in_db_and_return_state<D>(
    token_ticker: String,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_token_ticker_in_db(&state.db, &token_ticker)
        .map(|_| state)
}

fn put_empty_processed_tx_ids_in_db_and_return_state<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_processed_tx_ids_in_db(
        &state.db,
        &ProcessedTxIds::init()
    )
        .map(|_| state)
}

pub fn put_eos_chain_id_in_db_and_return_state<D>(
    chain_id: String,
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    put_eos_chain_id_in_db(
        &state.db,
        &convert_hex_to_checksum256(&chain_id)?,
    )
        .map(|_| state)
}

pub fn maybe_initialize_eos_core<D>(
    db: D,
    chain_id: String,
    token_ticker: String,
    account_name: String,
) -> Result<String>
    where D: DatabaseInterface
{
    trace!("✔ Maybe initializing EOS core...");
    match is_eos_core_initialized(&db) {
        true => {
            info!("✔ EOS core already initialized!");
            Ok("{eos_core_initialized:true}".to_string())
        }
        false => {
            info!("✔ Initializing core for EOS...");
            get_processed_tx_ids_from_db(&db)
                .and_then(|tx_ids|
                    start_eos_db_transaction(EosState::init(db, tx_ids))
                )
                .and_then(|state| {
                    put_empty_processed_tx_ids_in_db_and_return_state(state)
                })
                .and_then(|state|
                    put_eos_chain_id_in_db_and_return_state(
                        chain_id,
                        state
                    )
                )
                .and_then(|state|
                    put_eos_token_ticker_in_db_and_return_state(
                        token_ticker,
                        state
                    )
                )
                .and_then(|state|
                    put_eos_account_name_in_db_and_return_state(
                        account_name,
                        state
                    )
                )
                .and_then(end_eos_db_transaction)
                .and_then(get_eos_init_output)
        }
    }
}
