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
        increment_algo_account_nonce::maybe_increment_algo_account_nonce_and_return_eth_state,
        remove_old_eth_tail_block::maybe_remove_old_eth_tail_block_and_return_state,
        remove_receipts_from_canon_block::maybe_remove_receipts_from_eth_canon_block_and_return_state,
        update_eth_canon_block_hash::maybe_update_eth_canon_block_hash_and_return_state,
        update_eth_linker_hash::maybe_update_eth_linker_hash_and_return_state,
        update_eth_tail_block_hash::maybe_update_eth_tail_block_hash_and_return_state,
        update_latest_block_hash::maybe_update_latest_eth_block_hash_and_return_state,
        validate_block_in_state::validate_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    dictionaries::evm_algo::get_evm_algo_token_dictionary_and_add_to_eth_state,
    int_on_algo::{
        check_core_is_initialized::check_core_is_initialized_and_return_eth_state,
        int::{
            filter_submission_material::filter_submission_material_for_peg_in_events_in_state,
            filter_zero_value_tx_infos::filter_out_zero_value_tx_infos_from_state,
            get_int_output_json::get_int_output_json,
            parse_tx_infos::maybe_parse_tx_info_from_canon_block_and_add_to_state,
            sign_txs::maybe_sign_algo_txs_and_add_to_state,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an INT block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the INT
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an ALGO
/// transaction will be signed & returned to the caller.
pub fn submit_int_block_to_core<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting INT block to core...");
    parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(db))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
        .and_then(get_evm_algo_token_dictionary_and_add_to_eth_state)
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_tx_infos_from_state)
        .and_then(maybe_sign_algo_txs_and_add_to_state)
        .and_then(maybe_increment_algo_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_int_output_json)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_algorand::{AlgorandAddress, AlgorandGenesisId, AlgorandKeys};
    use serde_json::json;

    use super::*;
    use crate::{
        chains::{
            algo::algo_database_utils::AlgoDbUtils,
            eth::{
                core_initialization::initialize_eth_core::initialize_eth_core_with_vault_and_router_contracts_and_return_state,
                eth_chain_id::EthChainId,
                eth_crypto::eth_private_key::EthPrivateKey,
                eth_database_utils::{EthDbUtilsExt, EvmDbUtils},
                eth_utils::convert_hex_to_eth_address,
                vault_using_cores::VaultUsingCores,
            },
        },
        constants::{MAX_DATA_SENSITIVITY_LEVEL, MIN_DATA_SENSITIVITY_LEVEL},
        dictionaries::evm_algo::EvmAlgoTokenDictionary,
        int_on_algo::{
            int::get_int_output_json::IntOutput,
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
    fn should_submit_int_block_successfully() {
        let db = get_test_database();
        let int_submission_material = get_sample_contiguous_int_submission_json_strings();
        let int_init_block = int_submission_material[0].clone();
        let int_peg_in_block = int_submission_material[1].clone();
        let algo_submission_material = get_sample_contiguous_algo_submission_json_strings();
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
            &algo_submission_material[0],
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
        let int_db_utils = EvmDbUtils::new(&db);
        int_db_utils
            .put_eth_address_in_db(&int_db_utils.get_eth_address_key(), &int_address)
            .unwrap();
        int_db_utils.put_eth_private_key_in_db(&int_private_key).unwrap();
        assert_eq!(int_db_utils.get_public_eth_address_from_db().unwrap(), int_address);
        assert_eq!(int_db_utils.get_eth_private_key_from_db().unwrap(), int_private_key);

        // NOTE: Overwrite the ALGO address and private key since it's generated randomly above...
        let algo_db_utils = AlgoDbUtils::new(&db);
        let algo_address =
            AlgorandAddress::from_str("N4F4VB7GYZWL2RRTMQVMBKM5GKTKDTOHVB5PHGQYFB6XSXR3MRYIVOPTWE").unwrap();
        let algo_private_key = AlgorandKeys::from_bytes(
            &hex::decode("4c9a9699eedc1b7f62b679e375c32ed83159d22428892b7f4285dad2f550f558").unwrap(),
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_private_key_key").to_vec(),
            algo_private_key.to_bytes(),
            MAX_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        db.put(
            get_prefixed_db_key("algo_redeem_address_key").to_vec(),
            algo_address.to_bytes(),
            MIN_DATA_SENSITIVITY_LEVEL,
        )
        .unwrap();
        assert_eq!(algo_db_utils.get_redeem_address().unwrap(), algo_address);
        assert_eq!(algo_db_utils.get_algo_private_key().unwrap(), algo_private_key);

        // NOTE Save the token dictionary into the db...
        EvmAlgoTokenDictionary::new(vec![])
            .add_and_update_in_db(get_sample_evm_algo_dictionary_entry(), &db)
            .unwrap();

        // NOTE: Finally, submit the block containing the peg in!
        let output = submit_int_block_to_core(&db, &int_peg_in_block).unwrap();
        let expected_result_json = json!({
            "int_latest_block_number": 12221814,
            "algo_signed_transactions":[{
                "_id":"pint-on-algo-algo-0",
                "broadcast":false,
                "algo_tx_hash":"H6P2FZ5RGXNJZUPBJ23EB3T325BJ6SKY55NSHLOIQ6ML5ULJXFPA",
                "algo_signed_tx":"82a3736967c440c4da2d2dbcbf7bfb0b87b0d4ab172bb5fe61054a17b6d1ae903b9bc639b996fd125367a45dd09e05c6ddd16119900840613d5535971245d14d72967281d0ce0ba374786e8aa461616d74cd0539a461726376c42032a7dbdfcde7695d91ac438152fc908617ffbf9db94f843c250268e6fe21a0a0a3666565cd03e8a26676ce013b42d6a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa26c76ce013b46bea46e6f7465c48198c40103c403decaffc404ffffffffc42a307837316134343065653966613766393966623961363937653936656337383339623861313634336238c40403c38e67c43a474b54355858364e3435555633454e4d494f41564637455151594c37375034355846485949504246414a554f4e37524255435150583537325449c400c400a3736e64c4206f0bca87e6c66cbd4633642ac0a99d32a6a1cdc7a87af39a18287d795e3b6470a474797065a56178666572a478616964ce2a98f058",
                "algo_tx_amount":"1337",
                "algo_account_nonce":0, // FIXME
                "witnessed_timestamp":1650896969,
                "algo_tx_recipient":"GKT5XX6N45UV3ENMIOAVF7EQQYL77P45XFHYIPBFAJUON7RBUCQPX572TI",
                "host_token_address":"714666072",
                "originating_tx_hash":"0xb81f5564195f022f9812d5dfe80052afdfaf8cc86243a98b0dbdd887ef97bda7",
                "originating_address":"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
                "native_token_address":"0x4262d1f878d191fbc66dca73bad57309916b1412",
                "destination_chain_id":"0x03c38e67",
                "algo_latest_block_number":20642396,
                "broadcast_tx_hash":null,
                "broadcast_timestamp":null
            }]
        });
        let expected_result = IntOutput::from_str(&expected_result_json.to_string()).unwrap();
        let result = IntOutput::from_str(&output).unwrap();

        // NOTE: We don't assert against the timestamp because it's not deterministic!
        assert_eq!(result.int_latest_block_number, expected_result.int_latest_block_number);
        assert_eq!(
            result.algo_signed_transactions[0]._id,
            expected_result.algo_signed_transactions[0]._id
        );
        assert_eq!(
            result.algo_signed_transactions[0].broadcast,
            expected_result.algo_signed_transactions[0].broadcast
        );
        assert_eq!(
            result.algo_signed_transactions[0].algo_tx_hash,
            expected_result.algo_signed_transactions[0].algo_tx_hash
        );
        assert_eq!(
            result.algo_signed_transactions[0].algo_tx_amount,
            expected_result.algo_signed_transactions[0].algo_tx_amount
        );
        assert_eq!(
            result.algo_signed_transactions[0].host_token_address,
            expected_result.algo_signed_transactions[0].host_token_address
        );
        assert_eq!(
            result.algo_signed_transactions[0].originating_tx_hash,
            expected_result.algo_signed_transactions[0].originating_tx_hash
        );
        assert_eq!(
            result.algo_signed_transactions[0].originating_address,
            expected_result.algo_signed_transactions[0].originating_address
        );
        assert_eq!(
            result.algo_signed_transactions[0].native_token_address,
            expected_result.algo_signed_transactions[0].native_token_address
        );
        assert_eq!(
            result.algo_signed_transactions[0].algo_signed_tx,
            expected_result.algo_signed_transactions[0].algo_signed_tx
        );
        assert_eq!(
            result.algo_signed_transactions[0].algo_account_nonce,
            expected_result.algo_signed_transactions[0].algo_account_nonce
        );
        assert_eq!(
            result.algo_signed_transactions[0].algo_latest_block_number,
            expected_result.algo_signed_transactions[0].algo_latest_block_number
        );
        assert_eq!(
            result.algo_signed_transactions[0].broadcast_tx_hash,
            expected_result.algo_signed_transactions[0].broadcast_tx_hash
        );
        assert_eq!(
            result.algo_signed_transactions[0].broadcast_timestamp,
            expected_result.algo_signed_transactions[0].broadcast_timestamp
        );
        assert_eq!(
            result.algo_signed_transactions[0].algo_tx_recipient,
            expected_result.algo_signed_transactions[0].algo_tx_recipient
        );
        assert_eq!(
            result.algo_signed_transactions[0].destination_chain_id,
            expected_result.algo_signed_transactions[0].destination_chain_id
        );
    }
}
