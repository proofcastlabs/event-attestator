use crate::{
    chains::eth::{
        add_block_and_receipts_to_db::maybe_add_evm_block_and_receipts_to_db_and_return_state,
        check_parent_exists::check_for_parent_of_evm_block_in_state,
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_state::EthState,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        increment_eth_account_nonce::maybe_increment_eth_account_nonce_and_return_state,
        remove_old_eth_tail_block::maybe_remove_old_evm_tail_block_and_return_state,
        remove_receipts_from_canon_block::maybe_remove_receipts_from_evm_canon_block_and_return_state,
        update_eth_canon_block_hash::maybe_update_evm_canon_block_hash_and_return_state,
        update_eth_linker_hash::maybe_update_evm_linker_hash_and_return_state,
        update_eth_tail_block_hash::maybe_update_evm_tail_block_hash_and_return_state,
        update_latest_block_hash::maybe_update_latest_evm_block_hash_and_return_state,
        validate_block_in_state::validate_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    dictionaries::eth_evm::get_eth_evm_token_dictionary_from_db_and_add_to_eth_state,
    erc20_on_int::{
        check_core_is_initialized::check_core_is_initialized_and_return_eth_state,
        int::{
            account_for_fees::maybe_account_for_fees,
            divert_to_safe_address::{
                maybe_divert_txs_to_safe_address_if_destination_is_token_address,
                maybe_divert_txs_to_safe_address_if_destination_is_vault_address,
            },
            filter_submission_material::filter_submission_material_for_redeem_events_in_state,
            filter_zero_value_tx_infos::filter_out_zero_value_eth_tx_infos_from_state,
            get_int_output_json::get_evm_output_json,
            parse_tx_infos::maybe_parse_tx_info_from_canon_block_and_add_to_state,
            sign_txs::maybe_sign_eth_txs_and_add_to_evm_state,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an INT block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_int_block_to_core<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting INT block to core...");
    parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(&db))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
        .and_then(get_eth_evm_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(check_for_parent_of_evm_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_redeem_events_in_state)
        .and_then(maybe_add_evm_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_evm_block_hash_and_return_state)
        .and_then(maybe_update_evm_canon_block_hash_and_return_state)
        .and_then(maybe_update_evm_tail_block_hash_and_return_state)
        .and_then(maybe_update_evm_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_eth_tx_infos_from_state)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_vault_address)
        .and_then(maybe_sign_eth_txs_and_add_to_evm_state)
        .and_then(maybe_increment_eth_account_nonce_and_return_state)
        .and_then(maybe_remove_old_evm_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_evm_canon_block_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_evm_output_json)
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
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_utils::convert_hex_to_eth_address,
            vault_using_cores::VaultUsingCores,
        },
        dictionaries::eth_evm::EthEvmTokenDictionary,
        erc20_on_int::{
            int::get_int_output_json::IntOutput,
            test_utils::{
                get_sample_eth_init_block_json_string,
                get_sample_int_init_block_json_string,
                get_sample_peg_out_json_string,
                get_sample_token_dictionary_entry,
            },
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_submit_int_block_successfully() {
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
            &VaultUsingCores::Erc20OnInt,
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
        let address = convert_hex_to_eth_address("f4180c63f51a6063a1b18a1dd23823ea2e0e83c1").unwrap();
        let private_key = EthPrivateKey::from_slice(
            &hex::decode("f2dfe3a6e74ae140c2719e5a33df77eb2cb53a5dde5fb436822f80ad001ff93c").unwrap(),
        )
        .unwrap();
        let db_utils = EthDbUtils::new(&db);
        // NOTE: Overwrite the nonce since the test sample used the 3rd nonce...
        db_utils
            .put_eth_address_in_db(&db_utils.get_eth_address_key(), &address)
            .unwrap();
        db_utils.put_eth_private_key_in_db(&private_key).unwrap();
        assert_eq!(db_utils.get_public_eth_address_from_db().unwrap(), address,);
        assert_eq!(db_utils.get_eth_private_key_from_db().unwrap(), private_key,);
        // NOTE: Save the token dictionary in the db...
        EthEvmTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_token_dictionary_entry(), &db)
            .unwrap();
        // NOTE: Bring the ETH chain up to the block prior to the block containing a peg-in...
        let is_for_eth = false;
        reset_eth_chain(
            parse_eth_submission_material_and_put_in_state(
                &read_to_string("src/erc20_on_int/test_utils/int-before-peg-out-1-block.json").unwrap(),
                EthState::init(&db),
            )
            .unwrap(),
            confirmations,
            is_for_eth,
        )
        .unwrap();
        let submission_string = get_sample_peg_out_json_string();
        // NOTE: Finally, submit the block containting the peg out....
        let core_output = submit_int_block_to_core(db, &submission_string).unwrap();
        let expected_result_json = json!({
            "int_latest_block_number": 11572430,
            "eth_signed_transactions": [{
                "_id": "perc20-on-int-eth-0",
                "broadcast": false,
                "eth_tx_hash": "0x7032723dce91781fe9842d796be0ba82b3b3375d383e5464a8985a4bd33c2b17",
                "eth_tx_amount": "664",
                "eth_tx_recipient": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "witnessed_timestamp": 1638960988,
                "host_token_address": "0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "originating_tx_hash": "0x149b9d2522fa706c17218ace8816e853b687ad740940ee0f45255fe285d93b32",
                "originating_address": "0x0e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6",
                "native_token_address": "0xc63ab9437f5589e2c67e04c00a98506b43127645",
                "eth_signed_tx": "f901cb808504a817c8008306ddd094866e3fc7043efb8ff3a994f7d59f53fe045d4d7a80b9016422965469000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000c63ab9437f5589e2c67e04c00a98506b431276450000000000000000000000000000000000000000000000000000000000000298000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800069c322000000000000000000000000000000000000000000000000000000000000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f60000000000000000000000000000000000000000000000000000000000000003decaff00000000000000000000000000000000000000000000000000000000002ca0fb9b416930025b3028d81cab433c7582e19db81b017823c67e1bbf4d5afd87fba057eeb76610d644a08a4ef4a9ce4aa9107d31ffc0c3cd403f660ace283907f596",
                "any_sender_nonce": null,
                "eth_account_nonce": 0,
                "eth_latest_block_number": 9749607,
                "broadcast_tx_hash": null,
                "broadcast_timestamp": null,
                "any_sender_tx": null,
                "destination_chain_id": "0x00f34368",
            }]
        });
        let expected_result = IntOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = IntOutput::from_str(&core_output).unwrap();
        // NOTE: We don't assert against the timestamp because it's not deterministic!
        assert_eq!(result.int_latest_block_number, expected_result.int_latest_block_number);
        assert_eq!(
            result.eth_signed_transactions[0]._id,
            expected_result.eth_signed_transactions[0]._id
        );
        assert_eq!(
            result.eth_signed_transactions[0].broadcast,
            expected_result.eth_signed_transactions[0].broadcast
        );
        assert_eq!(
            result.eth_signed_transactions[0].eth_tx_hash,
            expected_result.eth_signed_transactions[0].eth_tx_hash
        );
        assert_eq!(
            result.eth_signed_transactions[0].eth_tx_amount,
            expected_result.eth_signed_transactions[0].eth_tx_amount
        );
        assert_eq!(
            result.eth_signed_transactions[0].host_token_address,
            expected_result.eth_signed_transactions[0].host_token_address
        );
        assert_eq!(
            result.eth_signed_transactions[0].originating_tx_hash,
            expected_result.eth_signed_transactions[0].originating_tx_hash
        );
        assert_eq!(
            result.eth_signed_transactions[0].originating_address,
            expected_result.eth_signed_transactions[0].originating_address
        );
        assert_eq!(
            result.eth_signed_transactions[0].native_token_address,
            expected_result.eth_signed_transactions[0].native_token_address
        );
        assert_eq!(
            result.eth_signed_transactions[0].eth_signed_tx,
            expected_result.eth_signed_transactions[0].eth_signed_tx
        );
        assert_eq!(
            result.eth_signed_transactions[0].any_sender_nonce,
            expected_result.eth_signed_transactions[0].any_sender_nonce
        );
        assert_eq!(
            result.eth_signed_transactions[0].eth_account_nonce,
            expected_result.eth_signed_transactions[0].eth_account_nonce
        );
        assert_eq!(
            result.eth_signed_transactions[0].eth_latest_block_number,
            expected_result.eth_signed_transactions[0].eth_latest_block_number
        );
        assert_eq!(
            result.eth_signed_transactions[0].broadcast_tx_hash,
            expected_result.eth_signed_transactions[0].broadcast_tx_hash
        );
        assert_eq!(
            result.eth_signed_transactions[0].broadcast_timestamp,
            expected_result.eth_signed_transactions[0].broadcast_timestamp
        );
        assert_eq!(
            result.eth_signed_transactions[0].any_sender_tx,
            expected_result.eth_signed_transactions[0].any_sender_tx
        );
        assert_eq!(
            result.eth_signed_transactions[0].destination_chain_id,
            expected_result.eth_signed_transactions[0].destination_chain_id,
        );
    }
}
