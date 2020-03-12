use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ProcessedTxIds,
        initialize_eos::is_eos_core_initialized::is_eos_core_initialized,
        eos_database_utils::{
            end_eos_db_transaction,
            start_eos_db_transaction,
            put_processed_tx_ids_in_db,
            get_processed_tx_ids_from_db,
        },
    },
};

fn get_eos_init_output<D>(
    _state: EosState<D>
) -> Result<String>
    where D: DatabaseInterface
{
    Ok("{eos_core_initialized:true}".to_string())
}

pub fn maybe_initialize_eos_core<D>(
    db: D,
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
                    put_processed_tx_ids_in_db(
                        &state.db,
                        &ProcessedTxIds::init()
                    )
                        .map(|_| state)
                })
                .and_then(end_eos_db_transaction)
                .and_then(get_eos_init_output)
        }
    }
}
