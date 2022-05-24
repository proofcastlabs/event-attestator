use crate::{
    chains::algo::{
        add_latest_algo_submission_material::add_latest_algo_submission_material_to_db_and_return_state,
        algo_database_transactions::{
            end_algo_db_transaction_and_return_state,
            start_algo_db_transaction_and_return_state,
        },
        algo_state::AlgoState,
        algo_submission_material::parse_algo_submission_material_and_put_in_state,
        check_parent_exists::check_parent_of_algo_block_in_state_exists,
        check_submitted_block_is_subsequent::check_submitted_block_is_subsequent_and_return_state,
        increment_eth_account_nonce::maybe_increment_eth_account_nonce_and_return_algo_state,
        remove_all_txs_from_submission_material_in_state::remove_all_txs_from_submission_material_in_state,
        remove_old_algo_tail_submission_material::maybe_remove_old_algo_tail_submission_material_and_return_state,
        remove_txs_from_canon_submission_material::maybe_remove_txs_from_algo_canon_submission_material_and_return_state,
        update_algo_canon_block_hash::maybe_update_algo_canon_block_hash_and_return_state,
        update_algo_linker_hash::maybe_update_algo_linker_hash_and_return_state,
        update_algo_tail_block_hash::maybe_update_algo_tail_block_hash_and_return_state,
    },
    dictionaries::evm_algo::get_evm_algo_token_dictionary_and_add_to_algo_state,
    int_on_algo::{
        algo::{
            add_relevant_txs_to_submission_material::add_relevant_validated_txs_to_submission_material_in_state,
            filter_zero_value_tx_infos::filter_out_zero_value_tx_infos_from_state,
            get_algo_output::get_algo_output,
            get_relevant_txs::get_relevant_asset_txs_from_submission_material_and_add_to_state,
            parse_tx_info::maybe_parse_tx_info_from_canon_block_and_add_to_state,
            sign_txs::maybe_sign_int_txs_and_add_to_algo_state,
            validate_relevant_txs::filter_out_invalid_txs_and_update_in_state,
        },
        check_core_is_initialized::check_core_is_initialized_and_return_algo_state,
    },
    traits::DatabaseInterface,
    types::Result,
};

/// Submit Algo Block To Core
///
/// The main submission pipeline. Submitting an Algorand block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ALGO
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain pertinent transactions to the redeem addres  the enclave is watching, an INT
/// transaction will be signed & returned to the caller.
pub fn submit_algo_block_to_core<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting ALGO block to core...");
    parse_algo_submission_material_and_put_in_state(block_json_string, AlgoState::init(db))
        .and_then(check_core_is_initialized_and_return_algo_state)
        .and_then(start_algo_db_transaction_and_return_state)
        .and_then(get_evm_algo_token_dictionary_and_add_to_algo_state)
        .and_then(check_parent_of_algo_block_in_state_exists)
        .and_then(check_submitted_block_is_subsequent_and_return_state)
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
        .and_then(maybe_sign_int_txs_and_add_to_algo_state)
        .and_then(maybe_increment_eth_account_nonce_and_return_algo_state)
        .and_then(maybe_remove_old_algo_tail_submission_material_and_return_state)
        .and_then(maybe_remove_txs_from_algo_canon_submission_material_and_return_state)
        .and_then(end_algo_db_transaction_and_return_state)
        .and_then(get_algo_output)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_algorand::{AlgorandAddress, AlgorandGenesisId};
    use serde_json::json;

    use super::*;
    use crate::{
        chains::{
            algo::algo_database_utils::AlgoDbUtils,
            eth::{
                core_initialization::initialize_eth_core::initialize_eth_core_with_vault_and_router_contracts_and_return_state,
                eth_chain_id::EthChainId,
                eth_crypto::eth_private_key::EthPrivateKey,
                eth_database_utils::{EthDbUtils, EthDbUtilsExt},
                eth_state::EthState,
                eth_utils::convert_hex_to_eth_address,
                vault_using_cores::VaultUsingCores,
            },
        },
        constants::MIN_DATA_SENSITIVITY_LEVEL,
        dictionaries::evm_algo::EvmAlgoTokenDictionary,
        int_on_algo::{
            algo::get_algo_output::AlgoOutput,
            maybe_initialize_algo_core,
            test_utils::{
                get_sample_contiguous_algo_submission_json_strings,
                get_sample_contiguous_int_submission_json_strings,
                get_sample_evm_algo_dictionary_entry,
                get_sample_router_address,
                get_sample_vault_address,
            },
        },
        test_utils::get_test_database,
        utils::get_prefixed_db_key,
    };

    #[test]
    fn should_submit_algo_block_successfully() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings();
        let algo_submission_material = get_sample_contiguous_algo_submission_json_strings();
        let int_init_block = int_submission_material[0].clone();
        let algo_init_block = algo_submission_material[0].clone();
        let algo_peg_in_block = algo_submission_material[2].clone();
        let router_address = get_sample_router_address();
        let vault_address = get_sample_vault_address();
        let int_confirmations = 0;
        let algo_confirmations = 1;
        let gas_price = 20_000_000_000;
        let algo_fee = 1000;

        // NOTE: Initialize the INT side of the core...
        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            gas_price,
            int_confirmations,
            EthState::init(&db),
            &vault_address,
            &router_address,
            &VaultUsingCores::IntOnAlgo,
        )
        .unwrap();

        // NOTE: Initialize the ALGO side of the core...
        maybe_initialize_algo_core(
            &db,
            &algo_init_block,
            &AlgorandGenesisId::Mainnet.to_string(),
            algo_fee,
            algo_confirmations,
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
            algo_address.to_bytes().unwrap(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry(), &db)
            .unwrap();

        submit_algo_block_to_core(&db, &algo_submission_material[1]).unwrap();

        // NOTE: Submit the block containing the peg in, though there will be no output due to 1 confirmations.
        let output = submit_algo_block_to_core(&db, &algo_peg_in_block).unwrap();
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

        // NOTE: We don't assert against the timestamp because it's not deterministic!
        assert_eq!(
            result.algo_latest_block_number,
            expected_result.algo_latest_block_number
        );
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
            result.int_signed_transactions[0].int_tx_recipient,
            expected_result.int_signed_transactions[0].int_tx_recipient
        );
        assert_eq!(
            result.int_signed_transactions[0].destination_chain_id,
            expected_result.int_signed_transactions[0].destination_chain_id
        );
    }
}