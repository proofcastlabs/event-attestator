use std::str::FromStr;

use common::{
    chains::eth::{
        add_block_and_receipts_to_db::maybe_add_eth_block_and_receipts_to_db_and_return_state,
        check_parent_exists::check_for_parent_of_eth_block_in_state,
        eth_submission_material::{
            parse_eth_submission_material_json_and_put_in_state,
            EthSubmissionMaterialJson,
            EthSubmissionMaterialJsons,
        },
        increment_eos_account_nonce::maybe_increment_eos_account_nonce_and_return_state,
        remove_old_eth_tail_block::maybe_remove_old_eth_tail_block_and_return_state,
        remove_receipts_from_canon_block::maybe_remove_receipts_from_eth_canon_block_and_return_state,
        update_eth_canon_block_hash::maybe_update_eth_canon_block_hash_and_return_state,
        update_eth_linker_hash::maybe_update_eth_linker_hash_and_return_state,
        update_eth_tail_block_hash::maybe_update_eth_tail_block_hash_and_return_state,
        update_latest_block_hash::maybe_update_latest_eth_block_hash_and_return_state,
        validate_block_in_state::validate_eth_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
        EthState,
    },
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};

use crate::int::{
    divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
    filter_receipts_in_state::filter_receipts_for_eos_on_int_eos_tx_info_in_state,
    filter_tx_info::{
        maybe_filter_out_int_tx_info_with_value_too_low_in_state,
        maybe_filter_out_zero_eos_asset_amounts_in_state,
    },
    get_int_output::{get_int_output, IntOutput, IntOutputs},
    parse_tx_info::maybe_parse_eth_tx_info_from_canon_block_and_add_to_state,
    sign_txs::maybe_sign_eos_txs_and_add_to_eth_state,
};

fn submit_int_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<IntOutput> {
    info!("✔ Submitting INT block to enclave...");
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(|state| state.get_eos_eth_token_dictionary_from_db_and_add_to_state())
        .and_then(validate_eth_block_in_state)
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_receipts_for_eos_on_int_eos_tx_info_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_eth_tx_info_from_canon_block_and_add_to_state)
        .and_then(maybe_filter_out_int_tx_info_with_value_too_low_in_state)
        .and_then(maybe_filter_out_zero_eos_asset_amounts_in_state)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(get_int_output)
}

/// # Submit INT Block to Core
///
/// The main submission pipeline. Submitting an INT block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the INT
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

#[cfg(all(test, feature = "non-validating"))] // NOTE: The test uses TELOS blocks, whose headers fail validation.
#[cfg(test)]
mod tests {
    use common::{
        chains::{
            eos::{
                core_initialization::initialize_eos_core::initialize_eos_core_inner,
                eos_crypto::eos_private_key::EosPrivateKey,
            },
            eth::{
                super::EthChainId,
                core_initialization::initialize_eth_core::initialize_eth_core_with_router_contract_and_return_state,
                eth_database_utils::{EthDbUtils, EthDbUtilsExt},
                EthState as IntState,
            },
        },
        test_utils::get_test_database,
    };
    use serde_json::json;

    use super::*;
    use crate::{
        int::get_int_output::IntOutput,
        test_utils::{
            get_contiguous_int_block_json_strs,
            get_sample_dictionary,
            get_sample_eos_init_block,
            get_sample_eos_private_key,
            get_sample_int_address,
            get_sample_int_private_key,
            get_sample_router_address,
        },
    };

    #[test]
    fn should_submit_int_block() {
        let db = get_test_database();
        let router_address = get_sample_router_address();

        // NOTE: Initialize the EOS core...
        let eos_chain_id = "4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11";
        let maybe_eos_account_name = Some("intoneostest");
        let maybe_eos_token_symbol = None;
        let eos_init_block = get_sample_eos_init_block();
        let is_native = true;
        initialize_eos_core_inner(
            &db,
            eos_chain_id,
            maybe_eos_account_name,
            maybe_eos_token_symbol,
            &eos_init_block,
            is_native,
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
        let is_native = false;
        initialize_eth_core_with_router_contract_and_return_state(
            &int_init_block,
            &EthChainId::Ropsten,
            int_gas_price,
            int_confirmations,
            IntState::init(&db),
            &router_address,
            is_native,
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
            IntOutput::from_str(&submit_int_block_to_core(&db, &contiguous_int_block_json_strs[1].clone()).unwrap())
                .unwrap();

        let expected_output = IntOutput::from_str(&json!({
            "int_latest_block_number": 11618227,
            "eos_signed_transactions": [{
                "_id": "peos-on-int-eos-0",
                "broadcast": false,
                "eos_tx_amount": "1.995000000 NAT",
                "int_tx_amount": "1995000000000000000",
                "eos_account_nonce": 0,
                "eos_tx_recipient": "intoneostest",
                "eos_tx_signature": "SIG_K1_K9NjQaUKbx48BGnA9zzed3XamVpGB5Gs3BtGJnjxHNUY7bhNYgJgFJ3vzEvkNFxNdxneCZ3rRe8F8kXsvriEZvzHe5xMBB",
                "witnessed_timestamp": 1656087134,
                "eos_serialized_tx": "32f0b562010002000000000000000190b1ca98aa49f37400000000644d99aa0190b1ca98aa49f37400000000a8ed3232593044c89ad68c55aec048e97600000000094e41540000000080a7823457a097c1380306decaffc0ffee040069c3222a30783731613434306565396661376639396662396136393765393665633738333962386131363433623800",
                "host_token_address": "0xa83446f219baec0b6fd6b3031c5a49a54543045b",
                "originating_tx_hash": "0xaa690ded7de895edfa62683325fefaa7cf207d9e4cdd873a3900cf2d8f45b934",
                "originating_address": "0x0e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6",
                "eos_latest_block_number": 222275383,
                "native_token_address": "ptestpout123",
                "broadcast_tx_hash": null,
                "broadcast_timestamp": null,
                "destination_chain_id": "0x028c7109"
            }
        ]}).to_string()).unwrap();

        // NOTE: And finally, we assert the output...
        let expected_num_txs = 1;
        assert_eq!(output.eos_signed_transactions.len(), expected_num_txs);
        let result = output.eos_signed_transactions[0].clone();
        let expected_result = expected_output.eos_signed_transactions[0].clone();
        assert_eq!(result, expected_result);
        // NOTE: The first four bytes/8 hex chars are an encoded timestamp,
        assert_eq!(result.eos_serialized_tx[8..], expected_result.eos_serialized_tx[8..]);
    }
}
