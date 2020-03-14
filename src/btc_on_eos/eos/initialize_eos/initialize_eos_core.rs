use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ProcessedTxIds,
        eos_database_utils::{
            end_eos_db_transaction,
            start_eos_db_transaction,
        },
        initialize_eos::{
            is_eos_core_initialized::is_eos_core_initialized,
            eos_init_utils::{
                get_eos_init_output,
                put_eos_chain_id_in_db_and_return_state,
                put_eos_account_name_in_db_and_return_state,
                put_eos_token_ticker_in_db_and_return_state,
                put_empty_processed_tx_ids_in_db_and_return_state,
            },
        },
    },
};

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
            start_eos_db_transaction(EosState::init(db, ProcessedTxIds::init()))
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
