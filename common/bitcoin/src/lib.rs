mod add_btc_block_to_db;
mod btc_block;
mod btc_constants;
mod btc_crypto;
mod btc_database_utils;
mod btc_debug_functions;
mod btc_enclave_state;
mod btc_metadata;
mod btc_recipients_and_amounts;
mod btc_state;
mod btc_submission_material;
mod btc_transaction;
mod btc_types;
mod btc_utils;
mod check_btc_parent_exists;
mod core_initialization;
mod deposit_address_info;
mod extract_utxos_from_p2pkh_txs;
mod extract_utxos_from_p2sh_txs;
mod extract_utxos_from_txs;
mod filter_btc_txs;
mod filter_deposit_address_info_hash_map;
mod filter_p2pkh_deposit_txs;
mod filter_p2sh_deposit_txs;
mod filter_utxos;
mod get_btc_block_in_db_format;
mod get_deposit_info_hash_map;
mod remove_old_btc_tail_block;
mod remove_tx_infos_from_canon_block;
mod save_utxos_to_db;
mod set_btc_anchor_block_hash;
mod set_btc_canon_block_hash;
mod set_btc_latest_block_hash;
mod set_flags;
mod update_btc_canon_block_hash;
mod update_btc_latest_block_hash;
mod update_btc_linker_hash;
mod update_btc_tail_block_hash;
mod utxo_manager;
mod validate_btc_block_header;
mod validate_btc_difficulty;
mod validate_btc_merkle_root;
mod validate_btc_proof_of_work;

pub mod test_utils;

pub use self::{
    add_btc_block_to_db::maybe_add_btc_block_to_db,
    btc_block::{parse_btc_block_and_id_and_put_in_state, BtcBlockAndId, BtcBlockInDbFormat},
    btc_constants::{
        BTC_NUM_DECIMALS,
        DEFAULT_BTC_SEQUENCE,
        MAX_NUM_OUTPUTS,
        MINIMUM_REQUIRED_SATOSHIS,
        PLACEHOLDER_BTC_ADDRESS,
        ZERO_HASH,
    },
    btc_crypto::BtcPrivateKey,
    btc_database_utils::{end_btc_db_transaction, BtcDatabaseKeysJson, BtcDbUtils},
    btc_debug_functions::{debug_set_btc_account_nonce, debug_set_btc_fee, debug_set_btc_utxo_nonce},
    btc_enclave_state::BtcEnclaveState,
    btc_metadata::ToMetadata,
    btc_recipients_and_amounts::{BtcRecipientAndAmount, BtcRecipientsAndAmounts},
    btc_state::BtcState,
    btc_submission_material::{
        parse_btc_submission_json_and_put_in_state,
        parse_submission_material_and_put_in_state,
        BtcSubmissionMaterialJson,
    },
    btc_transaction::create_signed_raw_btc_tx_for_n_input_n_outputs,
    btc_types::{BtcPubKeySlice, BtcTransactions},
    btc_utils::{
        convert_bytes_to_btc_pub_key_slice,
        create_unsigned_utxo_from_tx,
        get_hex_tx_from_signed_btc_tx,
        get_p2sh_redeem_script_sig,
        get_pay_to_pub_key_hash_script,
    },
    check_btc_parent_exists::check_for_parent_of_btc_block_in_state,
    core_initialization::maybe_initialize_btc_core,
    deposit_address_info::{validate_deposit_address_list_in_state, DepositAddressInfoJson, DepositInfoHashMap},
    extract_utxos_from_p2pkh_txs::{
        extract_utxos_from_p2pkh_tx,
        extract_utxos_from_p2pkh_txs,
        maybe_extract_utxos_from_p2pkh_txs_and_put_in_btc_state,
    },
    extract_utxos_from_p2sh_txs::maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
    extract_utxos_from_txs::extract_btc_utxo_from_btc_tx,
    filter_btc_txs::maybe_filter_out_btc_txs_with_too_many_outputs,
    filter_p2pkh_deposit_txs::filter_for_p2pkh_deposit_txs_including_change_outputs_and_add_to_state,
    filter_p2sh_deposit_txs::{filter_p2sh_deposit_txs, filter_p2sh_deposit_txs_and_add_to_state},
    filter_utxos::{filter_out_utxos_extant_in_db_from_state, filter_out_value_too_low_utxos_from_state},
    get_btc_block_in_db_format::create_btc_block_in_db_format_and_put_in_state,
    get_deposit_info_hash_map::{create_hash_map_from_deposit_info_list, get_deposit_info_hash_map_and_put_in_state},
    remove_old_btc_tail_block::maybe_remove_old_btc_tail_block,
    remove_tx_infos_from_canon_block::remove_tx_infos_from_canon_block_and_return_state,
    save_utxos_to_db::maybe_save_utxos_to_db,
    set_flags::set_any_sender_flag_in_state,
    update_btc_canon_block_hash::maybe_update_btc_canon_block_hash,
    update_btc_latest_block_hash::maybe_update_btc_latest_block_hash,
    update_btc_linker_hash::maybe_update_btc_linker_hash,
    update_btc_tail_block_hash::maybe_update_btc_tail_block_hash,
    utxo_manager::{
        debug_add_multiple_utxos,
        debug_clear_all_utxos,
        debug_consolidate_utxos,
        debug_consolidate_utxos_to_address,
        debug_get_child_pays_for_parent_btc_tx,
        debug_remove_utxo,
        get_all_utxos_as_json_string,
        get_enough_utxos_to_cover_total,
        get_utxo_constants_db_keys,
        save_utxos_to_db,
        set_utxo_balance_to_zero,
        BtcUtxoAndValue,
        BtcUtxosAndValues,
    },
    validate_btc_block_header::validate_btc_block_header_in_state,
    validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
    validate_btc_merkle_root::validate_btc_merkle_root,
    validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
};

#[macro_use]
extern crate paste;
#[macro_use]
extern crate common;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
