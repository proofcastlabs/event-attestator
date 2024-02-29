use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::{
    check_for_parent_of_evm_block_in_state,
    maybe_add_evm_block_and_receipts_to_db_and_return_state,
    maybe_increment_eth_account_nonce_and_return_state,
    maybe_remove_old_evm_tail_block_and_return_state,
    maybe_remove_receipts_from_evm_canon_block_and_return_state,
    maybe_update_evm_canon_block_hash_and_return_state,
    maybe_update_evm_linker_hash_and_return_state,
    maybe_update_evm_tail_block_hash_and_return_state,
    maybe_update_latest_evm_block_hash_and_return_state,
    parse_eth_submission_material_json_and_put_in_state,
    validate_evm_block_in_state,
    validate_receipts_in_state,
    EthState,
    EthSubmissionMaterialJson,
    EthSubmissionMaterialJsons,
};

use crate::int::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::{
        divert_tx_infos_to_safe_address_if_destination_is_router_address,
        divert_tx_infos_to_safe_address_if_destination_is_token_address,
        divert_tx_infos_to_safe_address_if_destination_is_vault_address,
        divert_tx_infos_to_safe_address_if_destination_is_zero_address,
    },
    filter_submission_material::filter_submission_material_for_redeem_events_in_state,
    filter_tx_info_with_no_erc20_transfer_event::filter_tx_info_with_no_erc20_transfer_event,
    filter_zero_value_tx_infos::filter_out_zero_value_eth_tx_infos_from_state,
    get_int_output_json::{get_evm_output_json, IntOutput, IntOutputs},
    parse_tx_infos::maybe_parse_tx_info_from_canon_block_and_add_to_state,
    sign_txs::maybe_sign_eth_txs_and_add_to_evm_state,
};

fn submit_int_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<IntOutput> {
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(validate_evm_block_in_state)
        .and_then(|state| state.get_eth_evm_token_dictionary_and_add_to_state())
        .and_then(check_for_parent_of_evm_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_redeem_events_in_state)
        .and_then(maybe_add_evm_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_evm_block_hash_and_return_state)
        .and_then(maybe_update_evm_canon_block_hash_and_return_state)
        .and_then(maybe_update_evm_tail_block_hash_and_return_state)
        .and_then(maybe_update_evm_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(filter_out_zero_value_eth_tx_infos_from_state)
        .and_then(maybe_account_for_fees)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(maybe_sign_eth_txs_and_add_to_evm_state)
        .and_then(maybe_increment_eth_account_nonce_and_return_state)
        .and_then(maybe_remove_old_evm_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_evm_canon_block_and_return_state)
        .and_then(get_evm_output_json)
}

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an INT block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_int_block_to_core<D: DatabaseInterface>(db: &D, block: &str) -> Result<String> {
    info!("✔ Submitting INT block to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthSubmissionMaterialJson::from_str(block))
        .and_then(|json| submit_int_block(db, &json))
        .and_then(|output| {
            db.end_transaction()?;
            Ok(output.to_string())
        })
}

/// # Submit INT Blocks to Core
///
/// Submit multiple INT blocks to the core.
pub fn submit_int_blocks_to_core<D: DatabaseInterface>(db: &D, blocks: &str) -> Result<String> {
    info!("✔ Batch submitting INT blocks to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthSubmissionMaterialJsons::from_str(blocks))
        .and_then(|jsons| {
            jsons
                .iter()
                .map(|block| submit_int_block(db, block))
                .collect::<Result<Vec<_>>>()
        })
        .map(IntOutputs::new)
        .and_then(|outputs| {
            db.end_transaction()?;
            Ok(outputs.to_output().to_string())
        })
}

// NOTE: Including origin tx details provisions more gas to txs, which changes tx encoding, hence
// cause test failures.
#[cfg(all(test, not(feature = "include-origin-tx-details")))]
mod tests {
    use std::fs::read_to_string;

    use common::{dictionaries::eth_evm::EthEvmTokenDictionary, test_utils::get_test_database};
    use common_chain_ids::EthChainId;
    use common_eth::{
        convert_hex_to_eth_address,
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        initialize_evm_core_with_no_contract_tx,
        parse_eth_submission_material_and_put_in_state,
        EthDbUtils,
        EthDbUtilsExt,
        EthPrivateKey,
        VaultUsingCores,
    };
    use common_eth_debug::reset_eth_chain;
    use serde_json::json;

    use super::*;
    use crate::{
        int::get_int_output_json::IntOutput,
        test_utils::{
            get_sample_eth_init_block_json_string,
            get_sample_int_init_block_json_string,
            get_sample_peg_out_json_string,
            get_sample_token_dictionary_entry,
        },
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
            true, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Initialize the INT side of the core...
        initialize_evm_core_with_no_contract_tx(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
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
                &read_to_string("src/test_utils/int-before-peg-out-1-block.json").unwrap(),
                EthState::init(&db),
            )
            .unwrap(),
            confirmations,
            is_for_eth,
        )
        .unwrap();
        let submission_string = get_sample_peg_out_json_string();
        // NOTE: Finally, submit the block containing the peg out....
        let core_output = submit_int_block_to_core(&db, &submission_string).unwrap();
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
        assert_eq!(result, expected_result)
    }
}

#[cfg(all(test, feature = "include-origin-tx-details"))]
mod tests {
    use std::fs::read_to_string;

    use common::{dictionaries::eth_evm::EthEvmTokenDictionary, test_utils::get_test_database};
    use common_chain_ids::EthChainId;
    use common_eth::{
        convert_hex_to_eth_address,
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        initialize_evm_core_with_no_contract_tx,
        parse_eth_submission_material_and_put_in_state,
        EthDbUtils,
        EthDbUtilsExt,
        EthPrivateKey,
        VaultUsingCores,
    };
    use common_eth_debug::reset_eth_chain;
    use serde_json::json;

    use super::*;
    use crate::{
        int::get_int_output_json::IntOutput,
        test_utils::{
            get_sample_eth_init_block_json_string,
            get_sample_int_init_block_json_string,
            get_sample_peg_out_json_string,
            get_sample_peg_out_with_origin_tx_details,
            get_sample_peg_out_with_origin_tx_details_init_block,
            get_sample_token_dictionary_entry_2,
        },
    };

    #[test]
    fn should_pass_through_origin_chain_tx_details() {
        let db = get_test_database();
        let router_address = convert_hex_to_eth_address("0x0e1c8524b1D1891B201ffC7BB58a82c96f8Fc4F6").unwrap();
        let vault_address = convert_hex_to_eth_address("0xe396757EC7E6aC7C8E5ABE7285dde47b98F22db8").unwrap();
        let confirmations = 0;
        let gas_price = 20_000_000_000;
        let eth_init_block = get_sample_eth_init_block_json_string();
        let int_init_block = get_sample_peg_out_with_origin_tx_details_init_block();
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
            true, // NOTE: is_native
        )
        .unwrap();
        // NOTE: Initialize the INT side of the core...
        initialize_evm_core_with_no_contract_tx(
            &int_init_block,
            &EthChainId::InterimChain,
            gas_price,
            confirmations,
            EthState::init(&db),
            false, // NOTE: is_native
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
            .add_and_update_in_db(get_sample_token_dictionary_entry_2(), &db)
            .unwrap();
        // NOTE: Bring the ETH chain up to the block prior to the block containing a peg-in...
        let is_for_eth = false;
        let submission_string = get_sample_peg_out_with_origin_tx_details();
        // NOTE: Finally, submit the block containing the peg out....
        let core_output = submit_int_block_to_core(&db, &submission_string).unwrap();
        let expected_result_json = json!({
            "int_latest_block_number": 22770077,
            "eth_signed_transactions": [{
                "_id": "perc20-on-int-eth-0",
                "broadcast": false,
                "eth_tx_hash": "0x6bb2b8c36e05553c6e23c549d159785476b2ee98192d438514168babaf4155f7",
                "eth_tx_amount": "306523355931",
                "eth_tx_recipient": "0xbb610bdd7cbd0d6e5e46516ac6dfe24cf79d4fa0",
                "witnessed_timestamp": 1638960988,
                "host_token_address": "0xa4aa256cd0288981d0e828282a39d7fca922b353",
                "originating_tx_hash": "0x0c679dc3e8c13ad1546dbf9adb9f463bc84e4c9a7201042cbc3b8b23f54b5237",
                "originating_address": "0x54d5a0638f23f0b89053f86eed60237bbc56e98c",
                "native_token_address": "0x15d4c048f83bd7e37d49ea4c83a07267ec4203da",
                "eth_signed_tx": "f8ca808504a817c8008303d09094e396757ec7e6ac7c8e5abe7285dde47b98f22db880b86483c09d42000000000000000000000000bb610bdd7cbd0d6e5e46516ac6dfe24cf79d4fa000000000000000000000000015d4c048f83bd7e37d49ea4c83a07267ec4203da000000000000000000000000000000000000000000000000000000475e373b1b2ca0a75d08ba041ad075d05135a88894fd179ee73218aec114c8494a33a698031b3ba0442c76c6983387f3ed265a38274e68fe038c2ad7030fb2fce21e80d53e60858a",
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
        assert_eq!(result, expected_result);
        let signed_tx = expected_result.eth_signed_transactions[0]
            .eth_signed_tx
            .clone()
            .unwrap();

        // NOTE: Assert that there's no mention of the interim chain in the tx.
        assert!(!signed_tx.contains("ffffffff"));
        // NOTE: Assert that the expected origin address exists in the tx.
        assert!(signed_tx.contains("bb610bdd7cbd0d6e5e46516ac6dfe24cf79d4fa0"));
    }
}
