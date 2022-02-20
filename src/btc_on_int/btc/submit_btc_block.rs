use crate::{
    btc_on_int::{
        btc::{
            divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
            filter_int_tx_infos::maybe_filter_out_value_too_low_btc_on_int_int_tx_infos_in_state,
            get_btc_output::{get_btc_output_and_put_in_state, get_btc_output_as_string},
            parse_tx_infos::parse_int_tx_infos_from_p2sh_deposits_and_add_to_state,
            sign_txs::maybe_sign_canon_block_txs,
        },
        check_core_is_initialized::check_core_is_initialized_and_return_btc_state,
    },
    chains::btc::{
        add_btc_block_to_db::maybe_add_btc_block_to_db,
        btc_block::parse_btc_block_and_id_and_put_in_state,
        btc_database_utils::{end_btc_db_transaction, start_btc_db_transaction},
        btc_state::BtcState,
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
    traits::DatabaseInterface,
    types::Result,
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
        .and_then(check_core_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(check_for_parent_of_btc_block_in_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(validate_deposit_address_list_in_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_int_tx_infos_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(filter_out_value_too_low_utxos_from_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(maybe_filter_out_value_too_low_btc_on_int_int_tx_infos_in_state)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
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

    use bitcoin::network::constants::Network as BtcNetwork;

    use super::*;
    use crate::{
        btc_on_int::{
            btc::get_btc_output::BtcOutput,
            int::initialize_int_core::init_int_core,
            test_utils::{
                get_sample_btc_submission_material_json_str_n,
                get_sample_eth_submission_material_json_str_n,
            },
        },
        chains::{
            btc::{
                btc_crypto::btc_private_key::BtcPrivateKey,
                btc_database_utils::BtcDbUtils,
                btc_submission_material::BtcSubmissionMaterial,
                core_initialization::initialize_btc_core::init_btc_core,
            },
            eth::{eth_state::EthState, eth_utils::convert_hex_to_eth_address},
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_submit_btc_blocks_to_core() {
        use simple_logger;
        simple_logger::init().unwrap();

        // Init the BTC core...
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

        // NOTE: Overwrite the private key and public address
        let pk = BtcPrivateKey::from_wif(btc_pk).unwrap();
        let address = pk.to_p2pkh_btc_address();
        btc_db_utils.put_btc_private_key_in_db(&pk).unwrap();
        btc_db_utils.put_btc_address_in_db(&address).unwrap();
        btc_db_utils
            .put_btc_pub_key_slice_in_db(&pk.to_public_key_slice())
            .unwrap();

        // Init the ETH core...
        let eth_block_0 = get_sample_eth_submission_material_json_str_n(0);
        let eth_state = EthState::init(&db);
        let eth_chain_id = 3;
        let eth_gas_price = 20_000_000_000;
        let eth_canon_to_tip_length = 3;
        let eth_address = convert_hex_to_eth_address("0x88d19e08Cd43bba5761c10c588b2A3D85C75041f").unwrap();
        init_int_core(
            eth_state,
            &eth_block_0,
            eth_chain_id,
            eth_gas_price,
            eth_canon_to_tip_length,
            &eth_address,
            &eth_address,
        )
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
        println!("here: {}", result_3);
        // FIXME asssert this output!
    }
}
