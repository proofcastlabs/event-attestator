use common::{
    chains::btc::{
        add_btc_block_to_db::maybe_add_btc_block_to_db,
        btc_block::parse_btc_block_and_id_and_put_in_state,
        btc_database_utils::{end_btc_db_transaction, start_btc_db_transaction},
        btc_submission_material::parse_btc_submission_json_and_put_in_state,
        check_btc_parent_exists::check_for_parent_of_btc_block_in_state,
        deposit_address_info::validate_deposit_address_list_in_state,
        extract_utxos_from_p2sh_txs::maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
        filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
        filter_utxos::filter_out_value_too_low_utxos_from_state,
        get_btc_block_in_db_format::create_btc_block_in_db_format_and_put_in_state,
        get_deposit_info_hash_map::get_deposit_info_hash_map_and_put_in_state,
        increment_eth_nonce::maybe_increment_eth_nonce_in_db,
        remove_old_btc_tail_block::maybe_remove_old_btc_tail_block,
        remove_tx_infos_from_canon_block::remove_tx_infos_from_canon_block_and_return_state,
        save_utxos_to_db::maybe_save_utxos_to_db,
        update_btc_canon_block_hash::maybe_update_btc_canon_block_hash,
        update_btc_latest_block_hash::maybe_update_btc_latest_block_hash,
        update_btc_linker_hash::maybe_update_btc_linker_hash,
        update_btc_tail_block_hash::maybe_update_btc_tail_block_hash,
        validate_btc_block_header::validate_btc_block_header_in_state,
        validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
        validate_btc_merkle_root::validate_btc_merkle_root,
        validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
    },
    core_type::CoreType,
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

use crate::btc::{
    divert_to_safe_address::{
        divert_tx_infos_to_safe_address_if_destination_is_router_address,
        divert_tx_infos_to_safe_address_if_destination_is_token_address,
        divert_tx_infos_to_safe_address_if_destination_is_zero_address,
    },
    filter_deposit_info_hash_map::filter_out_wrong_version_deposit_address_infos,
    filter_int_tx_infos::maybe_filter_out_value_too_low_btc_on_int_int_tx_infos_in_state,
    get_btc_output::{get_btc_output_and_put_in_state, get_btc_output_as_string},
    parse_tx_infos::parse_int_tx_infos_from_p2sh_deposits_and_add_to_state,
    sign_txs::maybe_sign_canon_block_txs,
};

/// # Submit BTC Block to Enclave
///
/// The main submission pipeline. Submitting a BTC block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the BTC
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a deposit to an address derived from the enclave's BTC public key, an INT
/// transaction will be signed & returned to the caller.
pub fn submit_btc_block_to_core<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting BTC block to enclave...");
    parse_btc_submission_json_and_put_in_state(block_json_string, BtcState::init(db))
        .and_then(parse_btc_block_and_id_and_put_in_state)
        .and_then(CoreType::check_core_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(check_for_parent_of_btc_block_in_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(validate_deposit_address_list_in_state)
        .and_then(filter_out_wrong_version_deposit_address_infos)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_int_tx_infos_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(filter_out_value_too_low_utxos_from_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(maybe_filter_out_value_too_low_btc_on_int_int_tx_infos_in_state)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(create_btc_block_in_db_format_and_put_in_state)
        .and_then(maybe_add_btc_block_to_db)
        .and_then(maybe_update_btc_latest_block_hash)
        .and_then(maybe_update_btc_canon_block_hash)
        .and_then(maybe_update_btc_tail_block_hash)
        .and_then(maybe_update_btc_linker_hash)
        .and_then(maybe_sign_canon_block_txs)
        .and_then(maybe_increment_eth_nonce_in_db)
        .and_then(maybe_remove_old_btc_tail_block)
        .and_then(get_btc_output_and_put_in_state)
        .and_then(remove_tx_infos_from_canon_block_and_return_state)
        .and_then(end_btc_db_transaction)
        .and_then(get_btc_output_as_string)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::{
        chains::{
            btc::{
                btc_crypto::btc_private_key::BtcPrivateKey,
                btc_database_utils::BtcDbUtils,
                btc_submission_material::BtcSubmissionMaterial,
                core_initialization::initialize_btc_core::init_btc_core,
                utxo_manager::utxo_database_utils::{get_first_utxo_and_value, get_utxo_nonce_from_db},
            },
            eth::{
                eth_crypto::{eth_private_key::EthPrivateKey, eth_transaction::EthTransaction},
                eth_database_utils::{EthDbUtils, EthDbUtilsExt},
                eth_utils::convert_hex_to_eth_address,
                EthState,
            },
        },
        metadata::{metadata_address::MetadataAddress, metadata_chain_id::MetadataChainId, Metadata},
        test_utils::get_test_database,
    };
    use ethereum_types::{Address as EthAddress, U256};
    use serde_json::json;

    use super::*;
    use crate::{
        btc::get_btc_output::BtcOutput,
        int::init_int_core,
        test_utils::{get_sample_btc_submission_material_json_str_n, get_sample_int_submission_material_json_str_n},
    };

    #[test]
    fn should_submit_btc_blocks_to_core() {
        // Init the BTC common...
        let btc_pk = "93GJ65qHNjGFHzQVTzEEAdBS7vMxe3XASfWE8RUASSfd3EtfmzP";
        let db = get_test_database();
        let btc_db_utils = BtcDbUtils::new(&db);
        let btc_state = BtcState::init(&db);
        let btc_fee = 100;
        let btc_difficulty = 1;
        let btc_network = "Testnet";
        let btc_canon_to_tip_length = 2;
        let btc_block_0 = get_sample_btc_submission_material_json_str_n(0);
        init_btc_core(
            btc_state,
            &btc_block_0,
            btc_fee,
            btc_difficulty,
            btc_network,
            btc_canon_to_tip_length,
        )
        .unwrap();

        // NOTE: Overwrite the BTC private key fields since they're randomly generated upon init.
        let btc_pk = BtcPrivateKey::from_wif(btc_pk).unwrap();
        let address = btc_pk.to_p2pkh_btc_address();
        btc_db_utils.put_btc_private_key_in_db(&btc_pk).unwrap();
        btc_db_utils.put_btc_address_in_db(&address).unwrap();
        btc_db_utils
            .put_btc_pub_key_slice_in_db(&btc_pk.to_public_key_slice())
            .unwrap();

        // Init the ETH common...
        let eth_block_0 = get_sample_int_submission_material_json_str_n(0);
        let eth_state = EthState::init(&db);
        let eth_chain_id = 3;
        let eth_gas_price = 20_000_000_000;
        let eth_canon_to_tip_length = 3;
        let ptoken_address_hex = "0x0f513aa8d67820787a8fdf285bfcf967bf8e4b8b";
        let ptoken_address = convert_hex_to_eth_address(ptoken_address_hex).unwrap();
        let router_address_hex = "0x88d19e08cd43bba5761c10c588b2a3d85c75041f";
        let router_address = convert_hex_to_eth_address(router_address_hex).unwrap();
        init_int_core(
            eth_state,
            &eth_block_0,
            eth_chain_id,
            eth_gas_price,
            eth_canon_to_tip_length,
            &ptoken_address,
            &router_address,
        )
        .unwrap();

        // NOTE: Overwrite the ETH private key fields since they're randomly generated upon init.
        let eth_db_utils = EthDbUtils::new(&db);
        let eth_pk_bytes = hex::decode("262e2a3a7fa5ae40ea04584f20b51fc3918b42e7dd89926b9f4e2196c8a032ba").unwrap();
        let eth_pk = EthPrivateKey::from_slice(&eth_pk_bytes).unwrap();
        eth_db_utils.put_eth_private_key_in_db(&eth_pk).unwrap();
        eth_db_utils
            .put_public_eth_address_in_db(&eth_pk.to_public_key().to_address())
            .unwrap();

        // NOTE: Submit first block, this one has a peg in in it.
        let btc_block_1 = get_sample_btc_submission_material_json_str_n(1);
        let result_1 = submit_btc_block_to_core(&db, &btc_block_1).unwrap();
        let expected_result_1 = BtcOutput::new(
            BtcSubmissionMaterial::from_str(&btc_block_1)
                .unwrap()
                .block_and_id
                .height,
            vec![],
        );
        assert_eq!(BtcOutput::from_str(&result_1).unwrap(), expected_result_1);

        let btc_block_2 = get_sample_btc_submission_material_json_str_n(2);
        let result_2 = submit_btc_block_to_core(&db, &btc_block_2).unwrap();
        let expected_result_2 = BtcOutput::new(
            BtcSubmissionMaterial::from_str(&btc_block_2)
                .unwrap()
                .block_and_id
                .height,
            vec![],
        );
        assert_eq!(BtcOutput::from_str(&result_2).unwrap(), expected_result_2);

        // NOTE: By now the block with the submission is the canon block, and hence a tx is signed.
        let btc_block_3 = get_sample_btc_submission_material_json_str_n(3);
        let result_3 = submit_btc_block_to_core(&db, &btc_block_3).unwrap();
        let originating_address = "2MvSnvTZFkNhrMH98zuKN4VyBsxz6dgRyJG".to_string();
        let int_tx_recipient = "0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC".to_string();
        let result = BtcOutput::from_str(&result_3).unwrap();
        let expected_result = BtcOutput::from_str(
            &json!({
                "btc_latest_block_number":2163205,
                "int_signed_transactions":[
                    {
                        "_id": "pbtc-on-int-int-0",
                        "broadcast": false,
                        "int_tx_hash": "0x778f3ff1e93436db2ee2f3b82c77e041d44f1c3408a3ea7bc46cf3d9730b9bf9",
                        "int_signed_tx": "f9036b808504a817c8008306ddd0940f513aa8d67820787a8fdf285bfcf967bf8e4b8b80b90304dcdc7dd000000000000000000000000088d19e08cd43bba5761c10c588b2a3d85c75041f00000000000000000000000000000000000000000000000000002ddb52308800000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000024003000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100018afeb20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001400069c3220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003c0ffee00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000023324d76536e76545a466b4e68724d4839387a754b4e3456794273787a36646752794a470000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a307866656466653236313665623336363163623866656432373832663566306363393164353964636163000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002aa0c9c90dd26eae43447b5ed57f46fab5122aa9a43d93489170dc5c023bd909c4c0a009379b580705dd06eed0b73f9d58063d3a3d6d2438a0b87cd51292c7ea827344",
                        "int_tx_amount":"50420000000000",
                        "int_account_nonce": 0,
                        "witnessed_timestamp": 1645454565,
                        "host_token_address": ptoken_address_hex,
                        "originating_tx_hash": "d3778ffe56b5c7f2f9613564eabb1812b5ceea3d199147c36d99ba955f5c634e",
                        "originating_address": originating_address,
                        "int_tx_recipient": int_tx_recipient,
                        "int_latest_block_number": 12000341,
                        "broadcast_tx_hash": null,
                        "broadcast_timestamp": null,
                        "destination_chain_id": "0x0069c322",
                    }
                ]
            }).to_string()
        ).unwrap();
        // NOTE: Assert the output...
        assert_eq!(result.btc_latest_block_number, expected_result.btc_latest_block_number);
        assert_eq!(result.int_signed_transactions.len(), 1);
        let tx_info = result.int_signed_transactions[0].clone();
        let expected_tx_info = expected_result.int_signed_transactions[0].clone();
        assert_eq!(tx_info._id, expected_tx_info._id);
        assert_eq!(tx_info.broadcast, expected_tx_info.broadcast);
        assert_eq!(tx_info.int_tx_hash, expected_tx_info.int_tx_hash);
        assert_eq!(tx_info.int_signed_tx, expected_tx_info.int_signed_tx);
        assert_eq!(tx_info.int_tx_amount, expected_tx_info.int_tx_amount);
        assert_eq!(tx_info.int_tx_recipient, expected_tx_info.int_tx_recipient);
        assert_eq!(tx_info.int_tx_recipient, expected_tx_info.int_tx_recipient);
        assert_eq!(tx_info.int_account_nonce, expected_tx_info.int_account_nonce);
        assert_eq!(tx_info.broadcast_tx_hash, expected_tx_info.broadcast_tx_hash);
        assert_eq!(tx_info.host_token_address, expected_tx_info.host_token_address);
        assert_eq!(tx_info.originating_tx_hash, expected_tx_info.originating_tx_hash);
        assert_eq!(tx_info.destination_chain_id, expected_tx_info.destination_chain_id);
        assert_eq!(
            tx_info.int_latest_block_number,
            expected_tx_info.int_latest_block_number
        );

        // NOTE: Decode the ETH transaction...
        let decoded_eth_tx = EthTransaction::from_bytes(&hex::decode(&tx_info.int_signed_tx).unwrap()).unwrap();
        let to = EthAddress::from_slice(&decoded_eth_tx.to);
        assert_eq!(to, ptoken_address);
        assert_eq!(decoded_eth_tx.nonce, U256::zero());
        assert_eq!(decoded_eth_tx.value, U256::zero());
        assert_eq!(decoded_eth_tx.gas_limit, U256::from(450_000));
        assert_eq!(decoded_eth_tx.gas_price, U256::from(eth_gas_price));

        // NOTE: Decode the data part of the transaction and assert that...
        let tx_data = decoded_eth_tx.data;
        let metadata = Metadata::decode_from_eth_v3(&tx_data[(4 + (32 * 5))..]).unwrap();
        let expected_metadata = Metadata::new_v3(
            &[0xc0, 0xff, 0xee],
            &MetadataAddress::new(&originating_address, &MetadataChainId::BitcoinTestnet).unwrap(),
            &MetadataAddress::new(&int_tx_recipient, &MetadataChainId::EthereumRopsten).unwrap(),
            Some(vec![]), // NOTE: Protocol options
            Some(vec![]), // NOTE: Protocol receipt
        );
        assert_eq!(metadata, expected_metadata);

        // NOTE: Assert that we indeed captured a UTXO and it's correct...
        let utxo_nonce = get_utxo_nonce_from_db(&db).unwrap();
        assert_eq!(utxo_nonce, 1);
        let utxo = get_first_utxo_and_value(&db).unwrap();
        assert_eq!(utxo.value, 5042);
    }
}
