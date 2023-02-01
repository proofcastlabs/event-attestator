use bitcoin::{blockdata::transaction::Transaction as BtcTransaction, network::constants::Network as BtcNetwork};

use crate::{
    btc_on_eos::eos::btc_tx_info::BtcOnEosBtcTxInfos,
    chains::btc::{
        btc_crypto::btc_private_key::BtcPrivateKey,
        btc_database_utils::BtcDbUtils,
        btc_recipients_and_amounts::{BtcRecipientAndAmount, BtcRecipientsAndAmounts},
        btc_transaction::create_signed_raw_btc_tx_for_n_input_n_outputs,
        utxo_manager::utxo_utils::get_enough_utxos_to_cover_total,
    },
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

fn get_address_and_amounts_from_btc_tx_infos(btc_tx_infos: &BtcOnEosBtcTxInfos) -> Result<BtcRecipientsAndAmounts> {
    info!("✔ Getting addresses & amounts from redeem params...");
    Ok(BtcRecipientsAndAmounts::new(
        btc_tx_infos
            .0
            .iter()
            .map(|params| {
                let recipient_and_amount = BtcRecipientAndAmount::new(&params.recipient[..], params.amount);
                info!(
                    "✔ Recipients & amount retrieved from redeem: {:?}",
                    recipient_and_amount
                );
                recipient_and_amount
            })
            .collect::<Result<Vec<BtcRecipientAndAmount>>>()?,
    ))
}

fn sign_txs_from_btc_tx_infos<D: DatabaseInterface>(
    btc_db_utils: &BtcDbUtils<D>,
    sats_per_byte: u64,
    btc_network: BtcNetwork,
    btc_tx_infos: &BtcOnEosBtcTxInfos,
    btc_address: &str,
    btc_private_key: &BtcPrivateKey,
) -> Result<BtcTransaction> {
    info!("✔ Getting correct amount of UTXOs...");
    debug!("✔ Network: {}", btc_network);
    debug!("✔ Satoshis per byte: {}", sats_per_byte);
    // FIXME This does not have the one tx per report implemented that the other cores do. As
    // such, this isn't using the MAX_NUM_OUTPUTS constants, which it should be doing.
    let utxos_and_values = get_enough_utxos_to_cover_total(
        btc_db_utils.get_db(),
        btc_tx_infos.sum(),
        btc_tx_infos.len() + 1, // NOTE: + 1 to account for the change output that's very likely to be required.
        sats_per_byte,
    )?;
    debug!("✔ Retrieved {} UTXOs!", utxos_and_values.len());
    info!("✔ Signing transaction...");
    create_signed_raw_btc_tx_for_n_input_n_outputs(
        sats_per_byte,
        get_address_and_amounts_from_btc_tx_infos(btc_tx_infos)?,
        btc_address,
        btc_private_key,
        utxos_and_values,
    )
}

pub fn maybe_sign_txs_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Maybe signing tx(s) from redeem params...");
    match &state.btc_on_eos_btc_tx_infos.len() {
        0 => {
            info!("✔ No redeem params in state ∴ not signing txs!");
            Ok(state)
        },
        _ => {
            info!("✔ Redeem params in state ∴ signing txs...");
            sign_txs_from_btc_tx_infos(
                &state.btc_db_utils,
                state.btc_db_utils.get_btc_fee_from_db()?,
                state.btc_db_utils.get_btc_network_from_db()?,
                &state.btc_on_eos_btc_tx_infos,
                &state.btc_db_utils.get_btc_address_from_db()?[..],
                &state.btc_db_utils.get_btc_private_key_from_db()?,
            )
            .and_then(|signed_tx| {
                debug!("✔ Signed transaction: {:?}", signed_tx);
                state.add_btc_on_eos_signed_txs(vec![signed_tx])
            })
        },
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::network::constants::Network as BtcNetwork;

    use super::*;
    use crate::{
        chains::{
            btc::{
                btc_database_utils::BtcDbUtils,
                btc_test_utils::{get_sample_p2sh_utxo_and_value_2, get_sample_p2sh_utxo_and_value_3},
                btc_utils::get_hex_tx_from_signed_btc_tx,
                utxo_manager::{
                    utxo_database_utils::{save_utxos_to_db, set_utxo_balance_to_zero},
                    utxo_types::BtcUtxosAndValues,
                },
            },
            eos::{eos_action_proofs::EosActionProof, eos_test_utils::get_sample_eos_submission_material_json_n},
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_get_correct_signed_btc_tx_3() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let db_utils = BtcDbUtils::new(&db);
        let sats_per_byte = 23;
        let btc_network = BtcNetwork::Testnet;
        let btc_address = "mwi6VyZUqwqdu1DtQMruV4UzEqJADZzj6n".to_string();
        let submission_material = get_sample_eos_submission_material_json_n(3);
        let action_proof = EosActionProof::from_json(&submission_material.action_proofs[0]).unwrap();
        let btc_tx_infos = BtcOnEosBtcTxInfos::from_action_proofs(&[action_proof]).unwrap();
        let utxo = get_sample_p2sh_utxo_and_value_2().unwrap();
        save_utxos_to_db(&db, &BtcUtxosAndValues::new(vec![utxo])).unwrap();
        let pk = BtcPrivateKey::from_slice(
            &hex::decode("2cc48e2f9066a0452e73cc7874f3fa8ba5ef705067d64bef627c686baa514336").unwrap(),
            btc_network,
        )
        .unwrap();
        pk.write_to_db(&db, &db_utils.get_btc_private_key_db_key()).unwrap();
        db_utils.put_btc_network_in_db(btc_network).unwrap();
        db_utils.put_btc_address_in_db(&btc_address).unwrap();
        let result =
            sign_txs_from_btc_tx_infos(&db_utils, sats_per_byte, btc_network, &btc_tx_infos, &btc_address, &pk)
                .unwrap();
        let result_hex = get_hex_tx_from_signed_btc_tx(&result);
        let expected_hex_result = "0100000001f8c70a7ecd6759cae01e96fca12993e5460d80a720d3fcffe2c95816ee29ef40000000008e473044022035b05463474179b1b7f4c3120aa38ba24a75504a80ca012875061a353c14b33a02200ba2428c42746ee6b4c632fb716ba1d9c1046e815164dafea6a0eb8a9ecf4b9c0145201729dce0b4e54e39610a95376a8bc96335fd93da68ae6aa27a62d4c282fb1ad3752102babc00d91bacf32c9ed07774e2be9aa3bc63296a858296c2fde390c339551f8dacffffffff0222160000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88acc43e0000000000001976a914b19d7011a6107944209d5ebf9ef31a0fdf3115f188ac00000000";
        assert_eq!(result_hex, expected_hex_result);
    }

    #[test]
    fn should_get_correct_signed_btc_tx_4() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let db_utils = BtcDbUtils::new(&db);
        let sats_per_byte = 23;
        let btc_network = BtcNetwork::Testnet;
        let btc_address = "mwi6VyZUqwqdu1DtQMruV4UzEqJADZzj6n".to_string();
        let submission_material = get_sample_eos_submission_material_json_n(4);
        let action_proof = EosActionProof::from_json(&submission_material.action_proofs[0]).unwrap();
        let btc_tx_infos = BtcOnEosBtcTxInfos::from_action_proofs(&[action_proof]).unwrap();
        let utxo = get_sample_p2sh_utxo_and_value_3().unwrap();
        save_utxos_to_db(&db, &BtcUtxosAndValues::new(vec![utxo])).unwrap();
        let pk = BtcPrivateKey::from_slice(
            &hex::decode("040cc91d329860197e118a1ea26b7ed7042de8f991d0600df9e482c367bb1c45").unwrap(),
            btc_network,
        )
        .unwrap();
        pk.write_to_db(&db, &db_utils.get_btc_private_key_db_key()).unwrap();
        db_utils.put_btc_network_in_db(btc_network).unwrap();
        db_utils.put_btc_address_in_db(&btc_address).unwrap();
        let result =
            sign_txs_from_btc_tx_infos(&db_utils, sats_per_byte, btc_network, &btc_tx_infos, &btc_address, &pk)
                .unwrap();
        let result_hex = get_hex_tx_from_signed_btc_tx(&result);
        let expected_hex_result = "0100000001d8baf6344ab19575fe40ad81e5ca1fa57345025e40de3975f7d58c7266e7b7a6000000008f483045022100f3ac5fb8676093a68135a9be21bcbeb118b65952b2c45b4552dd48617a46cb6902204d140fe6b8ddb6527840101d258d357d4f4363a2061dcf98e7ae35c1a655e16f014520d11539e07a521c78c20381c98cc546e3ccdd8a5c97f1cffe83ae5537f61a6e39752102f55e923c43236f553424b863b1d589b9b67add4d8c49aeca7e667c4772e7b942acffffffff02b3150000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88acdba00000000000001976a914b19d7011a6107944209d5ebf9ef31a0fdf3115f188ac00000000";
        assert_eq!(result_hex, expected_hex_result);
    }
}
