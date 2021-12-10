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
        increment_int_account_nonce::maybe_increment_int_account_nonce_and_return_eth_state,
        remove_old_eth_tail_block::maybe_remove_old_eth_tail_block_and_return_state,
        remove_receipts_from_canon_block::maybe_remove_receipts_from_eth_canon_block_and_return_state,
        update_eth_canon_block_hash::maybe_update_eth_canon_block_hash_and_return_state,
        update_eth_linker_hash::maybe_update_eth_linker_hash_and_return_state,
        update_eth_tail_block_hash::maybe_update_eth_tail_block_hash_and_return_state,
        update_latest_block_hash::maybe_update_latest_eth_block_hash_and_return_state,
        validate_block_in_state::validate_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    dictionaries::eth_evm::get_eth_evm_token_dictionary_from_db_and_add_to_eth_state,
    int_on_evm::{
        check_core_is_initialized::check_core_is_initialized_and_return_eth_state,
        int::{
            account_for_fees::maybe_account_for_fees,
            evm_tx_info::{
                filter_out_zero_value_evm_tx_infos_from_state,
                filter_submission_material_for_peg_in_events_in_state,
                maybe_divert_txs_to_safe_address_if_destination_is_evm_token_address,
                maybe_parse_tx_info_from_canon_block_and_add_to_state,
                maybe_sign_evm_txs_and_add_to_eth_state,
            },
            get_int_output_json::get_int_output_json,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an ETH block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_int_block_to_core<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting ETH block to core...");
    parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(&db))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
        .and_then(get_eth_evm_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_evm_tx_infos_from_state)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_evm_token_address)
        .and_then(maybe_sign_evm_txs_and_add_to_eth_state)
        .and_then(maybe_increment_int_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_int_output_json)
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use serde_json::json;

    use super::*;
    use crate::{
        chains::eth::{
            core_initialization::initialize_eth_core::{
                initialize_eth_core_with_vault_and_router_contracts_and_return_state,
                initialize_evm_core_with_no_contract_tx,
            },
            eth_chain_id::EthChainId,
            eth_crypto::eth_private_key::EthPrivateKey,
            eth_database_utils::{EthDbUtilsExt, EvmDbUtils},
            eth_utils::convert_hex_to_eth_address,
        },
        dictionaries::eth_evm::EthEvmTokenDictionary,
        int_on_evm::{
            int::get_int_output_json::IntOutput,
            test_utils::{
                get_sample_evm_init_block_json_string,
                get_sample_int_init_block_json_string,
                get_sample_router_address,
                get_sample_token_dictionary_entry,
                get_sample_vault_address,
            },
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_submit_int_block_successfully() {
        let db = get_test_database();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        // NOTE: Initialize the INT side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &get_sample_int_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
        )
        .unwrap();
        // NOTE: Initialize the EVM side of the core...
        initialize_evm_core_with_no_contract_tx(
            &get_sample_evm_init_block_json_string(),
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
        )
        .unwrap();
        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("0x969c70bccf47406e6d27ec91a12e66aedc7ef23e").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("f39d9bfba0555500b8b2c89cc46e90ae75fa80c23752ebae1ff31e3123d459dd").unwrap(),
            //&hex::decode("8d8f01916c70ff01244200f1768b9fb246158714ac05dc34cb6fca71798075a5").unwrap(),
        )
        .unwrap();
        let db_utils = EvmDbUtils::new(&db);
        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        // NOTE: Set the nonce to match that used during the test...
        let evm_nonce = 1;
        db_utils.put_eth_account_nonce_in_db(evm_nonce).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key);
        assert_eq!(db_utils.get_eth_account_nonce_from_db().unwrap(), evm_nonce);
        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry(), &db)
            .unwrap();
        let submission_string = read_to_string("src/int_on_evm/test_utils/peg-in-block-1.json").unwrap();
        // NOTE: Finally, submit the block containing the peg in....
        let output = submit_int_block_to_core(db, &submission_string).unwrap();
        let expected_result_json = json!({
            "int_latest_block_number": 11544952,
            "evm_signed_transactions": [{
                "_id": "pint-on-evm-evm-0",
                "broadcast": false,
                "evm_tx_hash": "0x57684b97e3ee2d6eafd50d07a1634afb2e3f7126ef0b9d906ea259cad52de1bf",
                "evm_tx_amount": "1337",
                "evm_tx_recipient": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "witnessed_timestamp": 1638896947,
                "host_token_address": "0xdd9f905a34a6c507c7d68384985905cf5eb032e9",
                "originating_tx_hash": "0x41bda64700fcd700e2c5ec7015da9b224f6c55e4859cb18ea164f2f826bede31",
                "originating_address": "0x0e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6",
                "native_token_address": "0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "evm_signed_tx": "f902ab018504a817c8008306ddd094dd9f905a34a6c507c7d68384985905cf5eb032e980b90244dcdc7dd0000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000053900000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000018002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100ffffffff000000000000000000000000000000000000000000000000000000000000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f60069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000003c0ffee00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa05ace98580e6a4a000fe8b81a0b39011cfe378f74f4cd8ea1d3cd669be8e13417a00c2da36ba23990a8b40ea1820e56bcd3f69ac89e43bd7608df229ea8df03c030",
                "any_sender_nonce": null,
                "evm_account_nonce": 0,
                "evm_latest_block_number": 11571205,
                "broadcast_tx_hash": null,
                "broadcast_timestamp": null,
                "any_sender_tx": null
            }]
        });
        let expected_result = IntOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = IntOutput::from_str(&output).unwrap();
        // NOTE: We don't assert against the timestamp because it's not deterministic!
        assert_eq!(result.int_latest_block_number, expected_result.int_latest_block_number);
        assert_eq!(
            result.evm_signed_transactions[0]._id,
            expected_result.evm_signed_transactions[0]._id
        );
        assert_eq!(
            result.evm_signed_transactions[0].broadcast,
            expected_result.evm_signed_transactions[0].broadcast
        );
        assert_eq!(
            result.evm_signed_transactions[0].evm_tx_hash,
            expected_result.evm_signed_transactions[0].evm_tx_hash
        );
        assert_eq!(
            result.evm_signed_transactions[0].evm_tx_amount,
            expected_result.evm_signed_transactions[0].evm_tx_amount
        );
        assert_eq!(
            result.evm_signed_transactions[0].host_token_address,
            expected_result.evm_signed_transactions[0].host_token_address
        );
        assert_eq!(
            result.evm_signed_transactions[0].originating_tx_hash,
            expected_result.evm_signed_transactions[0].originating_tx_hash
        );
        assert_eq!(
            result.evm_signed_transactions[0].originating_address,
            expected_result.evm_signed_transactions[0].originating_address
        );
        assert_eq!(
            result.evm_signed_transactions[0].native_token_address,
            expected_result.evm_signed_transactions[0].native_token_address
        );
        assert_eq!(
            result.evm_signed_transactions[0].evm_signed_tx,
            expected_result.evm_signed_transactions[0].evm_signed_tx
        );
        assert_eq!(
            result.evm_signed_transactions[0].any_sender_nonce,
            expected_result.evm_signed_transactions[0].any_sender_nonce
        );
        assert_eq!(
            result.evm_signed_transactions[0].evm_account_nonce,
            expected_result.evm_signed_transactions[0].evm_account_nonce
        );
        assert_eq!(
            result.evm_signed_transactions[0].evm_latest_block_number,
            expected_result.evm_signed_transactions[0].evm_latest_block_number
        );
        assert_eq!(
            result.evm_signed_transactions[0].broadcast_tx_hash,
            expected_result.evm_signed_transactions[0].broadcast_tx_hash
        );
        assert_eq!(
            result.evm_signed_transactions[0].broadcast_timestamp,
            expected_result.evm_signed_transactions[0].broadcast_timestamp
        );
        assert_eq!(
            result.evm_signed_transactions[0].any_sender_tx,
            expected_result.evm_signed_transactions[0].any_sender_tx
        );
    }
}
