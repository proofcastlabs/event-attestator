use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    check_core_is_initialized::check_core_is_initialized_and_return_eos_state,
    eos::{
        eos_state::EosState,
        get_eos_output::get_eos_output,
        save_btc_utxos_to_db::maybe_save_btc_utxos_to_db,
        sign_transactions::maybe_sign_txs_and_add_to_state,
        increment_signature_nonce::maybe_increment_signature_nonce,
        parse_redeem_params::maybe_parse_redeem_params_and_put_in_state,
        add_tx_ids_to_processed_list::maybe_add_tx_ids_to_processed_tx_ids,
        parse_submission_material::parse_submission_material_and_add_to_state,
        extract_utxos_from_btc_txs::maybe_extract_btc_utxo_from_btc_tx_in_state,
        filter_already_processed_txs::{
            maybe_filter_out_already_processed_tx_ids_from_state,
        },
        filter_redeem_params::{
            maybe_filter_value_too_low_redeem_params_in_state
        },
        eos_database_utils::{
            end_eos_db_transaction,
            start_eos_db_transaction,
            get_processed_tx_ids_from_db,
        },
    },
};

fn get_processed_tx_ids_and_add_to_state<D>( // TODO move to somewhere better
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    get_processed_tx_ids_from_db(&state.db)
        .and_then(|tx_ids| state.add_processed_tx_ids(tx_ids))
}

pub fn submit_eos_block_to_core<D>(
    db: D,
    block_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    parse_submission_material_and_add_to_state(block_json, EosState::init(db))
        .and_then(check_core_is_initialized_and_return_eos_state)
        .and_then(get_processed_tx_ids_and_add_to_state)
        .and_then(start_eos_db_transaction)
        //.and_then(validate_block_header_signatures)
        //.and_then(validate_irreversibility_proof)
        //.and_then(filter_invalid_action_proofs_from_state)
        //.and_then(filter_irrelevant_action_proofs_from_state)
        //.and_then(filter_duplicate_action_proofs_from_state)
        //.and_then(filter_already_processed_action_proofs_from_state)
        .and_then(maybe_parse_redeem_params_and_put_in_state)
        .and_then(maybe_filter_value_too_low_redeem_params_in_state)
        .and_then(maybe_filter_out_already_processed_tx_ids_from_state)
        .and_then(maybe_add_tx_ids_to_processed_tx_ids)
        .and_then(maybe_sign_txs_and_add_to_state)
        .and_then(maybe_increment_signature_nonce)
        .and_then(maybe_extract_btc_utxo_from_btc_tx_in_state)
        .and_then(maybe_save_btc_utxos_to_db)
        .and_then(end_eos_db_transaction)
        .and_then(get_eos_output)
}
