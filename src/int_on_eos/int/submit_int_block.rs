use crate::{
    chains::eth::{
        add_block_and_receipts_to_db::maybe_add_eth_block_and_receipts_to_db_and_return_state,
        check_parent_exists::check_for_parent_of_eth_block_in_state,
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_state::EthState,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        increment_eos_account_nonce::maybe_increment_eos_account_nonce_and_return_state,
        remove_old_eth_tail_block::maybe_remove_old_eth_tail_block_and_return_state,
        remove_receipts_from_canon_block::maybe_remove_receipts_from_eth_canon_block_and_return_state,
        update_eth_canon_block_hash::maybe_update_eth_canon_block_hash_and_return_state,
        update_eth_linker_hash::maybe_update_eth_linker_hash_and_return_state,
        update_eth_tail_block_hash::maybe_update_eth_tail_block_hash_and_return_state,
        update_latest_block_hash::maybe_update_latest_eth_block_hash_and_return_state,
        validate_block_in_state::validate_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    core_type::CoreType,
    dictionaries::eos_eth::get_eos_eth_token_dictionary_from_db_and_add_to_eth_state,
    int_on_eos::int::{
        divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
        filter_out_zero_tx_infos::filter_out_zero_value_eos_tx_infos_from_state,
        filter_submission_material::filter_submission_material_for_relevant_receipts_in_state,
        filter_tx_info_with_no_erc20_transfer_event::filter_tx_info_with_no_erc20_transfer_event,
        get_output_json::get_output_json,
        parse_tx_info::maybe_parse_eos_tx_info_from_canon_block_and_add_to_state,
        sign_txs::maybe_sign_eos_txs_and_add_to_eth_state,
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an INT block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the INT
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_int_block_to_core<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting INT block to enclave...");
    parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(db))
        .and_then(CoreType::check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
        .and_then(get_eos_eth_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_relevant_receipts_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_eos_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_eos_tx_infos_from_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_output_json)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_json::json;

    use super::*;
    use crate::{
        chains::{
            eos::{
                core_initialization::initialize_eos_core::initialize_eos_core_inner,
                eos_crypto::eos_private_key::EosPrivateKey,
            },
            eth::{
                core_initialization::initialize_eth_core::initialize_eth_core_with_vault_and_router_contracts_and_return_state,
                eth_chain_id::EthChainId,
                eth_database_utils::{EthDbUtils, EthDbUtilsExt},
                eth_state::EthState as IntState,
                vault_using_cores::VaultUsingCores,
            },
        },
        int_on_eos::{
            int::get_output_json::IntOnEosEosOutput,
            test_utils::{
                get_contiguous_int_block_json_strs,
                get_sample_dictionary,
                get_sample_eos_init_block,
                get_sample_eos_private_key,
                get_sample_int_address,
                get_sample_int_private_key,
                get_sample_router_address,
                get_sample_vault_address,
            },
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_submit_int_block() {
        let db = get_test_database();
        let vault_address = get_sample_vault_address();
        let router_address = get_sample_router_address();

        // NOTE: Initialize the EOS core...
        let eos_chain_id = "4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11";
        let maybe_eos_account_name = None;
        let maybe_eos_token_symbol = None;
        let eos_init_block = get_sample_eos_init_block();
        initialize_eos_core_inner(
            &db,
            eos_chain_id,
            maybe_eos_account_name,
            maybe_eos_token_symbol,
            &eos_init_block,
        )
        .unwrap();

        // NOTE: Overwrite the EOS private key since it's generated randomly above...
        let eos_pk = get_sample_eos_private_key();
        eos_pk.write_to_db(&db).unwrap();
        assert_eq!(EosPrivateKey::get_from_db(&db).unwrap(), eos_pk);

        // NOTE: Initialize the INT side of the core...
        let int_confirmations = 0;
        let int_gas_price = 20_000_000_000;
        let contiguous_int_block_json_strs = get_contiguous_int_block_json_strs();
        let int_init_block = contiguous_int_block_json_strs[0].clone();
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            int_gas_price,
            int_confirmations,
            IntState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnEos,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = get_sample_int_address();
        let int_private_key = get_sample_int_private_key();
        let int_db_utils = EthDbUtils::new(&db);
        int_db_utils
            .put_eth_address_in_db(&int_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        int_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(int_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(int_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Add the token dictionary to the db...
        let dictionary = get_sample_dictionary();
        dictionary.save_to_db(&db).unwrap();

        // NOTE: Submit the block with the peg in in it...
        let output =
            IntOnEosEosOutput::from_str(&submit_int_block_to_core(&db, &contiguous_int_block_json_strs[1]).unwrap())
                .unwrap();
        let expected_output = IntOnEosEosOutput::from_str(&json!({
            "eth_latest_block_number":12236006,
            "eos_signed_transactions":[{
                "_id":"pint-on-eos-eos-0",
                "broadcast":false,
                "eos_tx_amount":"0.13370000 IOE",
                "int_tx_amount":"133700000000000000",
                "eos_account_nonce":0,
                "eos_tx_recipient":"ptestpout111",
                "eos_tx_signature":"SIG_K1_KaXyH1xZpAyonUJYrPyqxLMFUhbnLrq7HUH4XmrNN7TH3Gd18WqPt3wgBjMd2hUSmvZsLCuhyqCDEuD1uTjpP5XY5VXuz6",
                "witnessed_timestamp":1651664234,
                "eos_serialized_tx":"5b078d6205253662b554000000000190b1ca98aa49f3740080c92671a531760190b1ca98aa49f37400000000a8ed32324f1042c89ad68c55ae9002cc000000000008494f450000000000350303decaff040069c3222a30783731613434306565396661376639396662396136393765393665633738333962386131363433623800",
                "host_token_address":"intoneostest",
                "originating_tx_hash":"0x0695f2980bc04b8da96406c79468f3e51dbb2a18c4bc0638cff055fa63f309f2",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "eos_latest_block_number":213498746,
                "destination_chain_id":"0x02e7261c",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            }]
        }).to_string()).unwrap();
        let expected_num_txs = 1;

        // NOTE: And finally, we assert the output...
        assert_eq!(output.eos_signed_transactions.len(), expected_num_txs);
        let result = output.eos_signed_transactions[0].clone();
        let expected_result = expected_output.eos_signed_transactions[0].clone();
        assert_eq!(result.eos_latest_block_number, expected_result.eos_latest_block_number);
        assert_eq!(result._id, expected_result._id);
        assert_eq!(result.broadcast, expected_result.broadcast);
        assert_eq!(result.eos_tx_amount, expected_result.eos_tx_amount);
        assert_eq!(result.int_tx_amount, expected_result.int_tx_amount);
        assert_eq!(result.eos_account_nonce, expected_result.eos_account_nonce);
        assert_eq!(result.eos_tx_recipient, expected_result.eos_tx_recipient);
        // NOTE: We drop the first 4 bytes since they're a timestamp & thus is not deterministic.
        assert_eq!(result.eos_serialized_tx[8..], expected_result.eos_serialized_tx[8..]);
        // NOTE: We also don't assert the sig since it's also timestamp based and thus not deterministic.
        assert_eq!(result.host_token_address, expected_result.host_token_address);
        assert_eq!(result.originating_tx_hash, expected_result.originating_tx_hash);
        assert_eq!(result.originating_address, expected_result.originating_address);
        assert_eq!(result.eos_latest_block_number, expected_result.eos_latest_block_number);
        assert_eq!(result.destination_chain_id, expected_result.destination_chain_id);
        assert_eq!(result.native_token_address, expected_result.native_token_address);
        assert_eq!(result.broadcast_tx_hash, expected_result.broadcast_tx_hash);
        assert_eq!(result.broadcast_timestamp, expected_result.broadcast_timestamp);
    }
}
