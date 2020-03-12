use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    check_core_is_initialized::check_core_is_initialized_and_return_eos_state,
    eos::{
        eos_state::EosState,
        parse_submission_material::parse_submission_material_and_add_to_state,
        eos_database_utils::{
            end_eos_db_transaction,
            start_eos_db_transaction,
        },
    },
};

pub fn submit_eos_block<D>(
    db: D,
    block_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    parse_submission_material_and_add_to_state(
        block_json,
        EosState::init(db),
    )
        // check enclave is initialized // TODO
        .and_then(start_eos_db_transaction)
        .and_then(check_core_is_initialized_and_return_eos_state)
        // validate block header signatures (skipped for now) // TODO
        // validate block is irreversible (assumed for now) // TODO
        //.and_then(filter_invalid_action_proofs_from_state) // FIXME
        //.and_then(filter_irrelevant_action_proofs_from_state) // FIXME
        // filter duplicate action proofs (serialized action duplicates)
        // filter action proof with nonces < last seen nonce
        // update last seen nonce (to greatest nonce in actions)
        // parse redeem params from proofs // TODO
        // sign btc transactions // TODO
        .and_then(end_eos_db_transaction)
        // get output // TODO
        .map(|_| "FIN".to_string())
}
