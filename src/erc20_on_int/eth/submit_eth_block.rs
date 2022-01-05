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
    erc20_on_int::{
        check_core_is_initialized::check_core_is_initialized_and_return_eth_state,
        eth::{
            account_for_fees::maybe_account_for_fees,
            divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_evm_token_address,
            filter_submission_material::filter_submission_material_for_peg_in_events_in_state,
            filter_zero_value_tx_infos::filter_out_zero_value_evm_tx_infos_from_state,
            get_eth_output_json::get_eth_output_json,
            parse_tx_info::maybe_parse_tx_info_from_canon_block_and_add_to_state,
            sign_txs::maybe_sign_int_txs_and_add_to_eth_state,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Submit ETH Block to Core
///
/// The main submission pipeline. Submitting an ETH block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_eth_block_to_core<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
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
        .and_then(maybe_sign_int_txs_and_add_to_eth_state)
        .and_then(maybe_increment_int_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_eth_output_json)
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use serde_json::json;

    use super::*;
    use crate::{
        chains::eth::{
            core_initialization::{
                initialize_eth_core::{
                    initialize_eth_core_with_vault_and_router_contracts_and_return_state,
                    initialize_evm_core_with_no_contract_tx,
                },
                reset_eth_chain::reset_eth_chain,
            },
            eth_chain_id::EthChainId,
            eth_crypto::eth_private_key::EthPrivateKey,
            eth_database_utils::{EthDbUtilsExt, EvmDbUtils},
            eth_utils::convert_hex_to_eth_address,
        },
        dictionaries::eth_evm::EthEvmTokenDictionary,
        erc20_on_int::{
            eth::get_eth_output_json::EthOutput,
            test_utils::{
                get_sample_eth_init_block_json_string,
                get_sample_int_init_block_json_string,
                get_sample_peg_in_1_submission_string,
                get_sample_token_dictionary_entry,
            },
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_submit_eth_block_successfully() {
        let db = get_test_database();
        let router_address = convert_hex_to_eth_address("0x0e1c8524b1D1891B201ffC7BB58a82c96f8Fc4F6").unwrap();
        let vault_address = convert_hex_to_eth_address("0x866e3fC7043EFb8ff3A994F7d59F53fe045d4d7A").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        let eth_init_block = get_sample_eth_init_block_json_string();
        let int_init_block = get_sample_int_init_block_json_string();
        // NOTE: Initialize the ETH side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &eth_init_block,
            &EthChainId::Rinkeby,
            gas_price,
            confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
        )
        .unwrap();
        // NOTE: Initialize the INT side of the core...
        initialize_evm_core_with_no_contract_tx(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
        )
        .unwrap();
        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let address = convert_hex_to_eth_address("8549cf9b30276305de31fa7533938e7ce366d12a").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("d22ecd05f55019604c5484bdb55d6c78c631cd7a05cc31781900ce356186617e").unwrap(),
        )
        .unwrap();
        let db_utils = EvmDbUtils::new(&db);
        // NOTE: Overwrite the nonce since the test sample used the 3rd nonce...
        let evm_nonce = 2;
        db_utils.put_eth_account_nonce_in_db(evm_nonce).unwrap();
        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address,);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key,);
        assert_eq!(db_utils.get_eth_account_nonce_from_db().unwrap(), evm_nonce);
        let is_for_eth = true;
        // NOTE Save the token dictionary into the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry(), &db)
            .unwrap();
        // NOTE: Bring the chain up to the block prior to the block containing a peg-in...
        reset_eth_chain(
            parse_eth_submission_material_and_put_in_state(
                &read_to_string("src/erc20_on_int/test_utils/eth-before-peg-in-1-block.json").unwrap(),
                EthState::init(&db),
            )
            .unwrap(),
            confirmations,
            is_for_eth,
        )
        .unwrap();
        let submission_string = get_sample_peg_in_1_submission_string();
        // NOTE: Finally, submit the block containting the peg in....
        let output = submit_eth_block_to_core(db, &submission_string).unwrap();
        let expected_result_json = json!({
            "eth_latest_block_number": 9750222,
            "int_signed_transactions": [
                {
                    "_id": "perc20-on-int-int-2",
                    "broadcast": false,
                    "int_tx_hash": "0x41bda64700fcd700e2c5ec7015da9b224f6c55e4859cb18ea164f2f826bede31",
                    "int_tx_amount": "1337",
                    "int_tx_recipient": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                    "witnessed_timestamp": 1638537255,
                    "host_token_address": "0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                    "originating_tx_hash": "0xf691d432fe940b2ecef70b6c9069ae124af9d160d761252d7ca570f5cd443dd4",
                    "originating_address": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                    "native_token_address": "0xc63ab9437f5589e2c67e04c00a98506b43127645",
                    "int_signed_tx": "f902ab028504a817c8008306ddd094a83446f219baec0b6fd6b3031c5a49a54543045b80b90244dcdc7dd00000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f600000000000000000000000000000000000000000000000000000000000005390000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000001800200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac0069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000003c0ffee00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0348b566813027d830760f46f5fad5794f69904b36dcf9ae28f281296e1947a43a06b5115551a2aa44ba5ba0515b21d84c8621ac6b60a665ea845ee23813d2350c2",
                    "any_sender_nonce": null,
                    "int_account_nonce": 2,
                    "int_latest_block_number": 11544277,
                    "broadcast_tx_hash": null,
                    "broadcast_timestamp": null,
                    "any_sender_tx": null
                }
            ]
        });
        let expected_result = EthOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = EthOutput::from_str(&output).unwrap();
        // NOTE: We don't assert against the timestamp because it's not deterministic!
        assert_eq!(result.eth_latest_block_number, expected_result.eth_latest_block_number);
        assert_eq!(
            result.int_signed_transactions[0]._id,
            expected_result.int_signed_transactions[0]._id
        );
        assert_eq!(
            result.int_signed_transactions[0].broadcast,
            expected_result.int_signed_transactions[0].broadcast
        );
        assert_eq!(
            result.int_signed_transactions[0].int_tx_hash,
            expected_result.int_signed_transactions[0].int_tx_hash
        );
        assert_eq!(
            result.int_signed_transactions[0].int_tx_amount,
            expected_result.int_signed_transactions[0].int_tx_amount
        );
        assert_eq!(
            result.int_signed_transactions[0].host_token_address,
            expected_result.int_signed_transactions[0].host_token_address
        );
        assert_eq!(
            result.int_signed_transactions[0].originating_tx_hash,
            expected_result.int_signed_transactions[0].originating_tx_hash
        );
        assert_eq!(
            result.int_signed_transactions[0].originating_address,
            expected_result.int_signed_transactions[0].originating_address
        );
        assert_eq!(
            result.int_signed_transactions[0].native_token_address,
            expected_result.int_signed_transactions[0].native_token_address
        );
        assert_eq!(
            result.int_signed_transactions[0].int_signed_tx,
            expected_result.int_signed_transactions[0].int_signed_tx
        );
        assert_eq!(
            result.int_signed_transactions[0].any_sender_nonce,
            expected_result.int_signed_transactions[0].any_sender_nonce
        );
        assert_eq!(
            result.int_signed_transactions[0].int_account_nonce,
            expected_result.int_signed_transactions[0].int_account_nonce
        );
        assert_eq!(
            result.int_signed_transactions[0].int_latest_block_number,
            expected_result.int_signed_transactions[0].int_latest_block_number
        );
        assert_eq!(
            result.int_signed_transactions[0].broadcast_tx_hash,
            expected_result.int_signed_transactions[0].broadcast_tx_hash
        );
        assert_eq!(
            result.int_signed_transactions[0].broadcast_timestamp,
            expected_result.int_signed_transactions[0].broadcast_timestamp
        );
        assert_eq!(
            result.int_signed_transactions[0].any_sender_tx,
            expected_result.int_signed_transactions[0].any_sender_tx
        );
    }
}
