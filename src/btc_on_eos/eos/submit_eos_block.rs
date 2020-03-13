use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    check_core_is_initialized::check_core_is_initialized_and_return_eos_state,
    eos::{
        eos_state::EosState,
        parse_redeem_params::maybe_parse_redeem_params_and_put_in_state,
        parse_submission_material::parse_submission_material_and_add_to_state,
        eos_database_utils::{
            end_eos_db_transaction,
            start_eos_db_transaction,
            get_processed_tx_ids_from_db,
        },
    },
};

pub fn submit_eos_block<D>(
    db: D,
    block_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    get_processed_tx_ids_from_db(&db)
        .and_then(|tx_ids|
            parse_submission_material_and_add_to_state(
                block_json,
                EosState::init(db, tx_ids),
            )
        )
        .and_then(check_core_is_initialized_and_return_eos_state)
        .and_then(start_eos_db_transaction)
        //.and_then(validate_block_header_signatures)
        //.and_then(validate_irreversibility_proof)
        //.and_then(filter_invalid_action_proofs_from_state)
        //.and_then(filter_irrelevant_action_proofs_from_state)
        //.and_then(filter_duplicate_action_proofs_from_state)
        //.and_then(filter_already_processed_action_proofs_from_state)
        //.and_then(add_tx_ids_to_processed_list)
        // sign btc transactions
        .and_then(maybe_parse_redeem_params_and_put_in_state)
        .and_then(end_eos_db_transaction)
        .map(|_| "FIN".to_string()) // TODO Output getter
}
