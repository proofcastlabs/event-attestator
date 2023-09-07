use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_algo::{
    add_latest_algo_submission_material_to_db_and_return_state,
    check_parent_of_algo_block_in_state_exists,
    maybe_remove_old_algo_tail_submission_material_and_return_state,
    maybe_remove_txs_from_algo_canon_submission_material_and_return_state,
    maybe_update_algo_canon_block_hash_and_return_state,
    maybe_update_algo_linker_hash_and_return_state,
    maybe_update_algo_tail_block_hash_and_return_state,
    maybe_update_latest_block_with_expired_participants_and_return_state,
    remove_all_txs_from_submission_material_in_state,
    AlgoState,
    AlgoSubmissionMaterial,
    AlgoSubmissionMaterials,
};

use crate::{
    algo::{
        add_relevant_txs_to_submission_material::add_relevant_validated_txs_to_submission_material_in_state,
        divert_to_safe_address::{
            divert_tx_infos_to_safe_address_if_destination_is_router_address,
            divert_tx_infos_to_safe_address_if_destination_is_token_address,
            divert_tx_infos_to_safe_address_if_destination_is_vault_address,
            divert_tx_infos_to_safe_address_if_destination_is_zero_address,
        },
        filter_zero_value_tx_infos::filter_out_zero_value_tx_infos_from_state,
        get_algo_output::{get_algo_output, AlgoOutput},
        get_relevant_txs::get_relevant_asset_txs_from_submission_material_and_add_to_state,
        maybe_increment_eth_account_nonce_and_return_algo_state,
        parse_tx_info::maybe_parse_tx_info_from_canon_block_and_add_to_state,
        sign_txs::maybe_sign_int_txs_and_add_to_algo_state,
        validate_relevant_txs::filter_out_invalid_txs_and_update_in_state,
    },
    token_dictionary::get_evm_algo_token_dictionary_and_add_to_algo_state,
};

fn submit_algo_block<D: DatabaseInterface>(db: &D, m: &AlgoSubmissionMaterial) -> Result<AlgoOutput> {
    AlgoState::init(db)
        .add_algo_submission_material(m)
        .and_then(get_evm_algo_token_dictionary_and_add_to_algo_state)
        .and_then(maybe_update_latest_block_with_expired_participants_and_return_state)
        .and_then(check_parent_of_algo_block_in_state_exists)
        .and_then(get_relevant_asset_txs_from_submission_material_and_add_to_state)
        .and_then(filter_out_invalid_txs_and_update_in_state)
        .and_then(remove_all_txs_from_submission_material_in_state)
        .and_then(add_relevant_validated_txs_to_submission_material_in_state)
        .and_then(add_latest_algo_submission_material_to_db_and_return_state)
        .and_then(maybe_update_algo_canon_block_hash_and_return_state)
        .and_then(maybe_update_algo_tail_block_hash_and_return_state)
        .and_then(maybe_update_algo_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_tx_infos_from_state)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(maybe_sign_int_txs_and_add_to_algo_state)
        .and_then(maybe_increment_eth_account_nonce_and_return_algo_state)
        .and_then(maybe_remove_old_algo_tail_submission_material_and_return_state)
        .and_then(maybe_remove_txs_from_algo_canon_submission_material_and_return_state)
        .and_then(get_algo_output)
}

/// Submit Algo Block To Core
///
/// The main submission pipeline. Submitting an Algorand block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ALGO
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain pertinent transactions to the redeem addres  the enclave is watching, an INT
/// transaction will be signed & returned to the caller.
pub fn submit_algo_block_to_core<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("submitting ALGO block to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| AlgoSubmissionMaterial::from_str(block_json_string))
        .and_then(|ref m| submit_algo_block(db, m))
        .and_then(|output| {
            db.end_transaction()?;
            Ok(output.to_string())
        })
}

/// # Submit ALGO Blocks to Core
///
/// Submit multiple ALGO blocks to the core.
pub fn submit_algo_blocks_to_core<D: DatabaseInterface>(db: &D, blocks: &str) -> Result<String> {
    info!("batch submitting ALGO blocks to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| db.start_transaction())
        .and_then(|_| AlgoSubmissionMaterials::from_str(blocks))
        .and_then(|ms| ms.iter().map(|m| submit_algo_block(db, m)).collect::<Result<Vec<_>>>())
        .map(AlgoOutput::from)
        .and_then(|outputs| {
            db.end_transaction()?;
            Ok(outputs.to_string())
        })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::{
        constants::MIN_DATA_SENSITIVITY_LEVEL,
        dictionaries::evm_algo::EvmAlgoTokenDictionary,
        test_utils::get_test_database,
        utils::get_prefixed_db_key,
    };
    use common_algo::AlgoDbUtils;
    use common_chain_ids::EthChainId;
    use common_eth::{
        convert_hex_to_eth_address,
        initialize_eth_core_with_vault_and_router_contracts_and_return_state,
        EthDbUtils,
        EthDbUtilsExt,
        EthPrivateKey,
        EthState,
        VaultUsingCores,
    };
    use rust_algorand::{AlgorandAddress, AlgorandGenesisId};
    use serde_json::json;
    use serial_test::serial;

    use super::*;
    use crate::{
        maybe_initialize_algo_core,
        test_utils::{
            get_sample_algo_batch_submission_string,
            get_sample_contiguous_algo_submission_json_strings_for_application_call_multi_peg_out,
            get_sample_contiguous_algo_submission_json_strings_for_application_call_peg_out,
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_1,
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_2,
            get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in,
            get_sample_evm_algo_dictionary_entry_1,
            get_sample_evm_algo_dictionary_entry_2,
            get_sample_router_address,
            get_sample_vault_address,
        },
    };

    fn assert_output(result: &AlgoOutput, expected_result: &AlgoOutput) {
        // NOTE: We don't assert against the timestamp because it's not deterministic!
        assert_eq!(
            result.algo_latest_block_number,
            expected_result.algo_latest_block_number
        );
        expected_result
            .int_signed_transactions
            .iter()
            .enumerate()
            .for_each(|(i, signed_tx)| {
                assert_eq!(
                    signed_tx._id,
                    result.int_signed_transactions[i]._id,
                    "\n{}",
                    format_args!("Wrong `_id` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.broadcast,
                    result.int_signed_transactions[i].broadcast,
                    "\n{}",
                    format_args!("Wrong `broadcast` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.int_tx_hash,
                    result.int_signed_transactions[i].int_tx_hash,
                    "\n{}",
                    format_args!("Wrong `int_tx_hash` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.int_tx_amount,
                    result.int_signed_transactions[i].int_tx_amount,
                    "\n{}",
                    format_args!("Wrong `int_tx_amount` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.host_token_address,
                    result.int_signed_transactions[i].host_token_address,
                    "\n{}",
                    format_args!("Wrong `host_token_address` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.originating_tx_hash,
                    result.int_signed_transactions[i].originating_tx_hash,
                    "\n{}",
                    format_args!("Wrong `originating_tx_hash` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.originating_address,
                    result.int_signed_transactions[i].originating_address,
                    "\n{}",
                    format_args!("Wrong `originating_address` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.native_token_address,
                    result.int_signed_transactions[i].native_token_address,
                    "\n{}",
                    format_args!("Wrong `native_token_address` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.int_signed_tx,
                    result.int_signed_transactions[i].int_signed_tx,
                    "\n{}",
                    format_args!("Wrong `int_signed_tx` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.int_account_nonce,
                    result.int_signed_transactions[i].int_account_nonce,
                    "\n{}",
                    format_args!("Wrong `int_account_nonce` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.int_latest_block_number,
                    result.int_signed_transactions[i].int_latest_block_number,
                    "\n{}",
                    format_args!("Wrong `int_latest_block_number` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.broadcast_tx_hash,
                    result.int_signed_transactions[i].broadcast_tx_hash,
                    "\n{}",
                    format_args!("Wrong `broadcast_tx_hash` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.broadcast_timestamp,
                    result.int_signed_transactions[i].broadcast_timestamp,
                    "\n{}",
                    format_args!("Wrong `broadcast_timestamp` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.int_tx_recipient,
                    result.int_signed_transactions[i].int_tx_recipient,
                    "\n{}",
                    format_args!("Wrong `int_tx_recipient` @ index: {}", i),
                );
                assert_eq!(
                    signed_tx.destination_chain_id,
                    result.int_signed_transactions[i].destination_chain_id,
                    "\n{}",
                    format_args!("Wrong `destination_chain_id` @ index: {}", i),
                );
            })
    }

    #[test]
    #[serial]
    fn should_submit_algo_block_with_asset_transfer_peg_out_successfully_1() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_1();
        let int_init_block = int_submission_material[0].clone();
        let algo_init_block = algo_submission_material[0].clone();
        let algo_peg_out_block = algo_submission_material[1].clone();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_init_block,
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils
            .put_eth_address_in_db(&eth_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        eth_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(eth_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(eth_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Submit the block containing the peg out, though there will be no output due to 1 confirmations.
        submit_algo_block_to_core(&db, &algo_peg_out_block).unwrap();

        // NOTE: Submit the next block to the core, which will result in a signed transaction.
        let output = submit_algo_block_to_core(&db, &algo_submission_material[2]).unwrap();
        let expected_result_json = json!({
            "algo_latest_block_number":20642398,
            "int_signed_transactions":[{
                "_id":"pint-on-algo-int-0",
                "broadcast":false,
                "int_tx_hash":"0xc376d379f30df9056fe84ce7e7b816e0cd983c91cbfb8d2ec17656c101482ca6",
                "int_tx_amount":"133700000000",
                "int_account_nonce":0,
                "int_tx_recipient":"0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC",
                "witnessed_timestamp":1650986069,
                "host_token_address":"714666072",
                "originating_tx_hash":"RHROWWML4P2ZWMJPOA7H4RYQJKIAXUNN7XM6TN24X2RRQBBFAEIQ",
                "originating_address":"GKT5XX6N45UV3ENMIOAVF7EQQYL77P45XFHYIPBFAJUON7RBUCQPX572TI",
                "destination_chain_id":"0x0069c322",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "int_signed_tx":"f9034b808504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902e422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b14120000000000000000000000000000000000000000000000000000001f21241900000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002400300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e670000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001400069c3220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003c0ffee0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a474b54355858364e3435555633454e4d494f41564637455151594c37375034355846485949504246414a554f4e37524255435150583537325449000000000000000000000000000000000000000000000000000000000000000000000000002a307866656466653236313665623336363163623866656432373832663566306363393164353964636163000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029a0d0ff7ad55cdfc290c578a32efa663a2824fea724d3957c0045b543a0dcbff782a022e90dc4bfaf66d8a0a7a1c0062776f6e2480e9866c36e52730907980cf51b88",
                "int_latest_block_number":12221813,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            }]
        });
        let expected_result = AlgoOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = AlgoOutput::from_str(&output).unwrap();
        assert_output(&result, &expected_result);
    }

    #[test]
    #[serial]
    fn should_submit_algo_block_with_asset_transfer_peg_out_successfully_2() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out_2();
        let int_init_block = int_submission_material[0].clone();
        let algo_init_block = algo_submission_material[0].clone();
        let algo_peg_out_block = algo_submission_material[1].clone();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_init_block,
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils
            .put_eth_address_in_db(&eth_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        eth_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(eth_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(eth_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("XXMOLMZYB5SMKNZP4PN4FRR5EX64BJURDZDYSQULMXS5SB354SDLBZX6HI").unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_2(), &db)
            .unwrap();

        // NOTE: Submit the block containing the peg out, though there will be no output due to 1 confirmations.
        submit_algo_block_to_core(&db, &algo_peg_out_block).unwrap();

        // NOTE: Submit the next block to the core, which will result in a signed transaction.
        let output = submit_algo_block_to_core(&db, &algo_submission_material[2]).unwrap();
        let expected_result_json = json!({
            "algo_latest_block_number":29285130,
            "int_signed_transactions":[{
                "_id":"pint-on-algo-int-0",
            "broadcast":false,
            "int_tx_hash":"0x9b44bbcf6590b359e9be271210c96433a7dd2f36a183f83c9006b23bb4b1acf9",
            "int_tx_amount":"1993896196000000000000",
            "int_account_nonce":0,
            "int_tx_recipient":"0xbA26A5FE275c52085901609D98Eb5d71480A4483",
            "witnessed_timestamp":1650986069,
            "host_token_address":"748211185",
            "originating_tx_hash":"DGQGIT64MMKLV4PP7PXVJK6OFZ6HGUTJZ7V7ZNGVRZKDIHLPBVFQ",
            "originating_address":"ZGVH5EWHRIQVNRDQKMODY6YWVWNXGH5E3DYYY4COX3KOESSUAGVQPWUOKE",
            "destination_chain_id":"0x005fe7f9",
            "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
            "int_signed_tx":"f9032b808504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902c422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b141200000000000000000000000000000000000000000000006c16de4a25aea44000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002200300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e67000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000120005fe7f900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a5a47564835455748524951564e5244514b4d4f445936595756574e5847483545334459595934434f58334b4f45535355414756515057554f4b45000000000000000000000000000000000000000000000000000000000000000000000000002a307862613236613566653237356335323038353930313630396439386562356437313438306134343833000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029a06dd168e2c277e36750ab654e1530611ba6e6731c2d4527f09ff8380d7af9b263a059927970f480082a6c373430e7703238b58d798456c4ee4841053248d34dd9d1",
            "int_latest_block_number":12221813,
            "broadcast_tx_hash":null,
            "broadcast_timestamp":null}]
        });
        let expected_result = AlgoOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = AlgoOutput::from_str(&output).unwrap();
        assert_output(&result, &expected_result);
    }

    #[test]
    #[serial]
    fn should_submit_algo_block_with_application_call_peg_out_successfully() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_application_call_peg_out();
        let int_init_block = int_submission_material[0].clone();
        let algo_init_block = algo_submission_material[0].clone();
        let algo_peg_out_block = algo_submission_material[1].clone();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_init_block,
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils
            .put_eth_address_in_db(&eth_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        eth_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(eth_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(eth_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Submit the block containing the peg out, though there will be no output due to 1 confirmations.
        submit_algo_block_to_core(&db, &algo_peg_out_block).unwrap();

        // NOTE: Submit the next block to the core, which will result in a signed transaction.
        let output = submit_algo_block_to_core(&db, &algo_submission_material[2]).unwrap();
        let expected_result_json = json!({
            "algo_latest_block_number": 21515432,
            "int_signed_transactions":[{
                "_id":"pint-on-algo-int-0",
                "broadcast":false,
                "int_tx_hash":"0x63514419a5a3aa8b8373e8acb926e0f88b6aacd5496e54940a7c550d193223e5",
                "int_tx_amount":"100000000000",
                "int_account_nonce":0,
                "int_tx_recipient":"0xc8D59c57B8C58Eac1622C7A639E10bF8B1E3DF9D",
                "witnessed_timestamp":1650986069,
                "host_token_address":"714666072",
                "originating_tx_hash":"3DZCWDYZDYCEBSPCVI4YMP47IUMU4UPWQYTT7FLN2P2CIJUPCGYQ",
                "originating_address":"E644GKJQW2YOJACA6DFT4OCHNQE6SJVC7K2ORLGZWFBAKRTAM44M63VHGA",
                "destination_chain_id":"0x00f34368",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "int_signed_tx":"f9032b808504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902c422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b1412000000000000000000000000000000000000000000000000000000174876e800000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002200300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e6700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a45363434474b4a515732594f4a41434136444654344f43484e514536534a5643374b324f524c475a574642414b5254414d34344d363356484741000000000000000000000000000000000000000000000000000000000000000000000000002a307863386435396335376238633538656163313632326337613633396531306266386231653364663964000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029a004af6aa128fe9f4503d119bc6b4a8bfce4de916a617868149d94523389790352a0352695ed32b73e3a1ae3bd4d020260e35158cc5e5d92b4d735506a9f2543b412",
                "int_latest_block_number":12221813,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            }]
        });
        let expected_result = AlgoOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = AlgoOutput::from_str(&output).unwrap();
        assert_output(&result, &expected_result);
    }

    #[test]
    #[serial]
    fn should_submit_algo_block_with_application_call_multi_peg_out() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_application_call_multi_peg_out();
        let int_init_block = int_submission_material[0].clone();
        let algo_init_block = algo_submission_material[0].clone();
        let algo_peg_out_block = algo_submission_material[1].clone();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_init_block,
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils
            .put_eth_address_in_db(&eth_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        eth_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(eth_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(eth_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Submit the block containing the peg out, though there will be no output due to 1 confirmations.
        submit_algo_block_to_core(&db, &algo_peg_out_block).unwrap();

        // NOTE: Submit the next block to the core, which will result in a signed transaction.
        let output = submit_algo_block_to_core(&db, &algo_submission_material[2]).unwrap();
        let expected_result_json = json!({
            "algo_latest_block_number": 21530959,
            "int_signed_transactions":[{
                "_id":"pint-on-algo-int-0",
                "broadcast":false,
                "int_tx_hash":"0x3cda08a19d9d4867eddf97282ddf1b8ab3c54c0d96be1f68dd99da1911a2a6b4",
                "int_tx_amount":"1000000000000000000",
                "int_account_nonce":0,
                "int_tx_recipient":"0xc8D59c57B8C58Eac1622C7A639E10bF8B1E3DF9D",
                "witnessed_timestamp":1654865303,
                "host_token_address":"714666072",
                "originating_tx_hash":"VCW6DXNYMRANYVXS2KXYXPW5IKQFGIETGZI5EKEZSVGHXPHDBNWQ",
                "originating_address":"E644GKJQW2YOJACA6DFT4OCHNQE6SJVC7K2ORLGZWFBAKRTAM44M63VHGA",
                "destination_chain_id":"0x00f34368",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "int_signed_tx":"f9032b808504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902c422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b14120000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002200300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e6700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a45363434474b4a515732594f4a41434136444654344f43484e514536534a5643374b324f524c475a574642414b5254414d34344d363356484741000000000000000000000000000000000000000000000000000000000000000000000000002a30786338643539633537623863353865616331363232633761363339653130626638623165336466396400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0ed94e2e30206a9755acc7f48f432dd82d2f8cae56095bb5af45bda7a5ea8a08ca02b8874159698d0bfbac7fb3a761e52f0cb6726a5a2f881e518e5ea7a168dc64c",
                "int_latest_block_number":12221813,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            },{
                "_id":"pint-on-algo-int-1",
                "broadcast":false,
                "int_tx_hash":"0xc10fb6b53e0ae32495045759548b89933e52de8521bb7ac34a5f6ce789aab017",
                "int_tx_amount":"500000000000000000",
                "int_account_nonce":1,
                "int_tx_recipient":"0xc8D59c57B8C58Eac1622C7A639E10bF8B1E3DF9D",
                "witnessed_timestamp":1654865303,
                "host_token_address":"714666072",
                "originating_tx_hash":"VCW6DXNYMRANYVXS2KXYXPW5IKQFGIETGZI5EKEZSVGHXPHDBNWQ",
                "originating_address":"E644GKJQW2YOJACA6DFT4OCHNQE6SJVC7K2ORLGZWFBAKRTAM44M63VHGA",
                "destination_chain_id":"0x00f34368",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "int_signed_tx":"f9032b018504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902c422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b141200000000000000000000000000000000000000000000000006f05b59d3b20000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002200300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e6700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a45363434474b4a515732594f4a41434136444654344f43484e514536534a5643374b324f524c475a574642414b5254414d34344d363356484741000000000000000000000000000000000000000000000000000000000000000000000000002a30786338643539633537623863353865616331363232633761363339653130626638623165336466396400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0be435d9f04ec861dc6e19cf71437d8198f682313a8bcc26aee4f1a4bd4746acfa073aac0e2df515122acf925c5528c0c748468394606dd7e91e96fb1a14a821507",
                "int_latest_block_number":12221813,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            },{
                "_id":"pint-on-algo-int-2",
                "broadcast":false,
                "int_tx_hash":"0x99769d20ae179547fe8dcb98067df166890179ddfe527c0f6b924951cfc4c101",
                "int_tx_amount":"300000000000000000",
                "int_account_nonce":2,
                "int_tx_recipient":
                    "0xc8D59c57B8C58Eac1622C7A639E10bF8B1E3DF9D",
                    "witnessed_timestamp":1654865303,
                    "host_token_address":"714666072",
                    "originating_tx_hash":"VCW6DXNYMRANYVXS2KXYXPW5IKQFGIETGZI5EKEZSVGHXPHDBNWQ",
                    "originating_address":"E644GKJQW2YOJACA6DFT4OCHNQE6SJVC7K2ORLGZWFBAKRTAM44M63VHGA",
                    "destination_chain_id":"0x00f34368",
                    "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                    "int_signed_tx":"f9032b028504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902c422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b14120000000000000000000000000000000000000000000000000429d069189e0000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002200300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e6700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a45363434474b4a515732594f4a41434136444654344f43484e514536534a5643374b324f524c475a574642414b5254414d34344d363356484741000000000000000000000000000000000000000000000000000000000000000000000000002a30786338643539633537623863353865616331363232633761363339653130626638623165336466396400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0a6aead1c76514b73b25e3e36972d74a4c4b5fbf1e33ebf1b0f5a648e01411a51a00519c1a7c3d6d5ea1fc053c0ff73702893f56cb4ce7a3a724bfe09205208313d",
                    "int_latest_block_number":12221813,
                    "broadcast_tx_hash":null,
                    "broadcast_timestamp":null
            }]
        });
        let expected_result = AlgoOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = AlgoOutput::from_str(&output).unwrap();
        assert_output(&result, &expected_result);
    }

    #[test]
    #[serial]
    fn should_batch_submit_algo_blocks_to_core() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in();
        let algo_submission_material =
            get_sample_contiguous_algo_submission_json_strings_for_application_call_multi_peg_out();
        let int_init_block = int_submission_material[0].clone();
        let algo_init_block = algo_submission_material[0].clone();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;
        let app_id = 1337;

        // NOTE: Initialize the INT side of the core...
        let is_native = true;
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
            is_native,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_init_block,
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
            app_id,
        )
        .unwrap();

        // NOTE: Overwrite the INT address & private key since it's generated randomly above...
        let int_address = convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap();
        let int_private_key = EthPrivateKey::from_slice(
            &hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap(),
        )
        .unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils
            .put_eth_address_in_db(&eth_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        eth_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(eth_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(eth_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry_1(), &db)
            .unwrap();

        // NOTE: Create the submission material for a batch submission
        let batch_submission_material = get_sample_algo_batch_submission_string();

        // NOTE: Submit the next two blocks to the core, which will result in a signed transaction.
        let output = submit_algo_blocks_to_core(&db, &batch_submission_material).unwrap();

        let expected_result_json = json!({
            "algo_latest_block_number": 21530959,
            "int_signed_transactions":[{
                "_id":"pint-on-algo-int-0",
                "broadcast":false,
                "int_tx_hash":"0x3cda08a19d9d4867eddf97282ddf1b8ab3c54c0d96be1f68dd99da1911a2a6b4",
                "int_tx_amount":"1000000000000000000",
                "int_account_nonce":0,
                "int_tx_recipient":"0xc8D59c57B8C58Eac1622C7A639E10bF8B1E3DF9D",
                "witnessed_timestamp":1654865303,
                "host_token_address":"714666072",
                "originating_tx_hash":"VCW6DXNYMRANYVXS2KXYXPW5IKQFGIETGZI5EKEZSVGHXPHDBNWQ",
                "originating_address":"E644GKJQW2YOJACA6DFT4OCHNQE6SJVC7K2ORLGZWFBAKRTAM44M63VHGA",
                "destination_chain_id":"0x00f34368",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "int_signed_tx":"f9032b808504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902c422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b14120000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002200300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e6700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a45363434474b4a515732594f4a41434136444654344f43484e514536534a5643374b324f524c475a574642414b5254414d34344d363356484741000000000000000000000000000000000000000000000000000000000000000000000000002a30786338643539633537623863353865616331363232633761363339653130626638623165336466396400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0ed94e2e30206a9755acc7f48f432dd82d2f8cae56095bb5af45bda7a5ea8a08ca02b8874159698d0bfbac7fb3a761e52f0cb6726a5a2f881e518e5ea7a168dc64c",
                "int_latest_block_number":12221813,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            },{
                "_id":"pint-on-algo-int-1",
                "broadcast":false,
                "int_tx_hash":"0xc10fb6b53e0ae32495045759548b89933e52de8521bb7ac34a5f6ce789aab017",
                "int_tx_amount":"500000000000000000",
                "int_account_nonce":1,
                "int_tx_recipient":"0xc8D59c57B8C58Eac1622C7A639E10bF8B1E3DF9D",
                "witnessed_timestamp":1654865303,
                "host_token_address":"714666072",
                "originating_tx_hash":"VCW6DXNYMRANYVXS2KXYXPW5IKQFGIETGZI5EKEZSVGHXPHDBNWQ",
                "originating_address":"E644GKJQW2YOJACA6DFT4OCHNQE6SJVC7K2ORLGZWFBAKRTAM44M63VHGA",
                "destination_chain_id":"0x00f34368",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "int_signed_tx":"f9032b018504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902c422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b141200000000000000000000000000000000000000000000000006f05b59d3b20000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002200300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e6700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a45363434474b4a515732594f4a41434136444654344f43484e514536534a5643374b324f524c475a574642414b5254414d34344d363356484741000000000000000000000000000000000000000000000000000000000000000000000000002a30786338643539633537623863353865616331363232633761363339653130626638623165336466396400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0be435d9f04ec861dc6e19cf71437d8198f682313a8bcc26aee4f1a4bd4746acfa073aac0e2df515122acf925c5528c0c748468394606dd7e91e96fb1a14a821507",
                "int_latest_block_number":12221813,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            },{
                "_id":"pint-on-algo-int-2",
                "broadcast":false,
                "int_tx_hash":"0x99769d20ae179547fe8dcb98067df166890179ddfe527c0f6b924951cfc4c101",
                "int_tx_amount":"300000000000000000",
                "int_account_nonce":2,
                "int_tx_recipient":
                    "0xc8D59c57B8C58Eac1622C7A639E10bF8B1E3DF9D",
                    "witnessed_timestamp":1654865303,
                    "host_token_address":"714666072",
                    "originating_tx_hash":"VCW6DXNYMRANYVXS2KXYXPW5IKQFGIETGZI5EKEZSVGHXPHDBNWQ",
                    "originating_address":"E644GKJQW2YOJACA6DFT4OCHNQE6SJVC7K2ORLGZWFBAKRTAM44M63VHGA",
                    "destination_chain_id":"0x00f34368",
                    "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                    "int_signed_tx":"f9032b028504a817c8008306ddd094e0806ce04978224e27c6bb10e27fd30a7785ae9d80b902c422965469000000000000000000000000ec1700a39972482d5db20e73bb3ffe6829b0c1020000000000000000000000004262d1f878d191fbc66dca73bad57309916b14120000000000000000000000000000000000000000000000000429d069189e0000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002200300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010003c38e6700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003a45363434474b4a515732594f4a41434136444654344f43484e514536534a5643374b324f524c475a574642414b5254414d34344d363356484741000000000000000000000000000000000000000000000000000000000000000000000000002a30786338643539633537623863353865616331363232633761363339653130626638623165336466396400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0a6aead1c76514b73b25e3e36972d74a4c4b5fbf1e33ebf1b0f5a648e01411a51a00519c1a7c3d6d5ea1fc053c0ff73702893f56cb4ce7a3a724bfe09205208313d",
                    "int_latest_block_number":12221813,
                    "broadcast_tx_hash":null,
                    "broadcast_timestamp":null
            }]
        });
        let expected_result = AlgoOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = AlgoOutput::from_str(&output).unwrap();
        assert_output(&result, &expected_result);
    }
}
