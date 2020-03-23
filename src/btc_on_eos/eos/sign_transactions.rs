use bitcoin::{
    network::constants::Network as BtcNetwork,
    blockdata::transaction::Transaction as BtcTransaction,
};
use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    utxo_manager::utxo_database_utils::get_utxo_and_value,
    eos::{
        eos_state::EosState,
        eos_types::RedeemParams,
    },
    btc::{
        btc_transaction::create_signed_raw_btc_tx_for_n_input_n_outputs,
        btc_utils::{
            calculate_btc_tx_fee,
            get_total_value_of_utxos_and_values,
        },
        btc_database_utils::{
            get_btc_fee_from_db,
            get_btc_network_from_db,
            get_btc_address_from_db,
            get_btc_private_key_from_db,
        },
        btc_types::{
            BtcUtxoAndValue,
            BtcUtxosAndValues,
            BtcRecipientAndAmount,
            BtcRecipientsAndAmounts,
        },
    },
};

fn sum_redeem_params(
    redeem_params: &Vec<RedeemParams>
) -> u64 {
    info!("✔ Summing redeem param amounts...");
    redeem_params
        .iter()
        .map(|params| params.amount)
        .sum()
}

fn get_enough_utxos_to_cover_total<D>(
    db: &D,
    required_btc_amount: u64,
    num_outputs: usize,
    sats_per_byte: u64,
    mut inputs: Vec<BtcUtxoAndValue>,
) -> Result<BtcUtxosAndValues>
    where D: DatabaseInterface
{
    info!("✔ Getting UTXO from db...");
    get_utxo_and_value(db)
        .and_then(|utxo_and_value| {
            debug!("✔ Retrieved UTXO of value: {}", utxo_and_value.value);
            let fee = calculate_btc_tx_fee(
                inputs.len() + 1,
                num_outputs,
                sats_per_byte
            );
            let total_cost = fee + required_btc_amount;
            inputs.push(utxo_and_value);
            let total_utxo_value = get_total_value_of_utxos_and_values(&inputs);
            debug!(
               "✔ Calculated fee for {} input(s) & {} output(s): {} Satoshis",
               inputs.len(),
               num_outputs,
               fee
           );
            debug!("✔ Fee + required value of tx: {} Satoshis", total_cost);
            debug!("✔ Current total UTXO value: {} Satoshis", total_utxo_value);
            match total_cost > total_utxo_value {
                true => {
                    trace!("✔ UTXOs do not cover fee + amount, need another!");
                    get_enough_utxos_to_cover_total(
                        db,
                        required_btc_amount,
                        num_outputs,
                        sats_per_byte,
                        inputs,
                    )
                }
                false => {
                    trace!("✔ UTXO(s) covers fee and required amount!");
                    Ok(inputs)
                }
            }
        })
}

fn get_address_and_amounts_from_redeem_params(
    redeem_params: &Vec<RedeemParams>,
) -> BtcRecipientsAndAmounts {
    info!("✔ Getting addresses & amounts from redeem params...");
    redeem_params
        .iter()
        .map(|params| {
            let recipient_and_amount = BtcRecipientAndAmount::new(
                &params.recipient[..],
                params.amount,
            );
            info!(
                "✔ Recipients & amount retrieved from redeem: {:?}",
                recipient_and_amount
            );
            recipient_and_amount
         })
        .flatten()
        .collect::<BtcRecipientsAndAmounts>()
}

fn sign_txs_from_redeem_params<D>(
    db: &D,
    sats_per_byte: u64,
    btc_network: BtcNetwork,
    redeem_params: &Vec<RedeemParams>,
) -> Result<BtcTransaction>
    where D: DatabaseInterface
{
    info!("✔ Getting correct amount of UTXOs...");
    debug!("✔ Network: {}", btc_network);
    debug!("✔ Satoshis per byte: {}", sats_per_byte);
    let utxos_and_values = get_enough_utxos_to_cover_total(
        db,
        sum_redeem_params(&redeem_params),
        redeem_params.len(),
        sats_per_byte,
        Vec::new(),
    )?;
    debug!("✔ Retrieved {} UTXOs!", utxos_and_values.len());
    info!("✔ Signing transaction...");
    create_signed_raw_btc_tx_for_n_input_n_outputs(
        sats_per_byte,
        get_address_and_amounts_from_redeem_params(&redeem_params),
        &get_btc_address_from_db(db)?[..],
        get_btc_private_key_from_db(db)?,
        utxos_and_values,
    )
}

pub fn maybe_sign_txs_and_add_to_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe signing tx(s) from redeem params...");
    match &state.redeem_params.len() {
        0 => {
            info!("✔ No redeem params in state ∴ not signing txs!");
            Ok(state)
        }
        _ => {
            info!("✔ Redeem params in state ∴ signing txs...");
            sign_txs_from_redeem_params(
                &state.db,
                get_btc_fee_from_db(&state.db)?,
                get_btc_network_from_db(&state.db)?,
                &state.redeem_params,
            )
                .and_then(|signed_tx| {
                    #[cfg(feature="debug")] {
                        debug!("✔ Signed transaction: {:?}", signed_tx);
                    }
                    state.add_signed_txs(vec![signed_tx])
                })
        },
    }
}

#[cfg(test)]
mod tests {
    /*
    use super::*;

    use bitcoin::network::constants::Network as BtcNetwork;
    use crate::btc_on_eos::{
        test_utils::get_test_database,
        utxo_manager::utxo_database_utils::save_utxos_to_db,
        btc::{
            btc_constants::BTC_PRIVATE_KEY_DB_KEY,
            btc_utils::get_hex_tx_from_signed_btc_tx,
            btc_crypto::btc_private_key::BtcPrivateKey,
            btc_test_utils::{
                get_sample_p2sh_utxo_and_value_2,
                get_sample_p2sh_utxo_and_value_3,
                get_sample_p2sh_utxo_and_value_4,
            },
            btc_database_utils::{
                put_btc_network_in_db,
                put_btc_address_in_db,
            },
        },
        eos::{
            parse_redeem_params::parse_redeem_params_from_actions_data,
            eos_types::{
                ActionData,
                ActionProof,
                ActionParams,
            },
            eos_test_utils::{
                get_sample_eos_submission_material_json_n,
            },
        },
    };
    */

    /* FIXME Clean these tests up!
    #[test]
    fn should_test_failing_p2sh_tx_1() { // FIXME!
        let db = get_test_database();
        let sats_per_byte = 23;
        let btc_network = BtcNetwork::Testnet;
        let btc_address = "mtQpPJ2yqTN7qxLrRnTSJ8CpbD1BXjM2oo"
            .to_string();
        let submission_material = get_sample_eos_submission_material_json_n(3);
        let action_proof = ActionProof::from_json(
            &submission_material
                .action_proofs[0]
        ).unwrap();
        let action_params = ActionParams::from_json(
            &submission_material
                .action_proofs[0]
                .action_json
                .data
        ).unwrap();
        let action_data = ActionData::new(action_proof, action_params);
        let redeem_params = parse_redeem_params_from_actions_data(
            &vec![action_data],
        ).unwrap();
        let utxo = get_sample_p2sh_utxo_and_value_2()
            .unwrap();
        save_utxos_to_db(&db, &vec![utxo])
            .unwrap();
        let pk = BtcPrivateKey::from_slice(
            &hex::decode(
        "2cc48e2f9066a0452e73cc7874f3fa8ba5ef705067d64bef627c686baa514336"
            ).unwrap(),
            btc_network.clone(),
        ).unwrap();
        pk
            .write_to_database(&db, &BTC_PRIVATE_KEY_DB_KEY.to_vec())
            .unwrap();
        put_btc_network_in_db(&db, &btc_network)
            .unwrap();
        put_btc_address_in_db(&db, &btc_address)
            .unwrap();
        let result = sign_txs_from_redeem_params(
            &db,
            sats_per_byte,
            btc_network,
            &redeem_params,
        ).unwrap();

        let result_hex = get_hex_tx_from_signed_btc_tx(&result);
        let expected_hex_result = "0100000001f8c70a7ecd6759cae01e96fca12993e5460d80a720d3fcffe2c95816ee29ef40000000008e47304402200f8c0e9cc392f540b9b389b635bf78629b64c447bade313ced8d207a91ca063a02200bace8c78b21ce2e8a11dc07cddb04a75336e54f806cbd987d4420f50b863a5d0145201729dce0b4e54e39610a95376a8bc96335fd93da68ae6aa27a62d4c282fb1ad3752102babc00d91bacf32c9ed07774e2be9aa3bc63296a858296c2fde390c339551f8dacffffffff02c2140000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88ac8f3b0000000000001976a9148d70896a446fef5fe85db1597d6dce8b72f796e088ac00000000";
        assert_eq!(result_hex, expected_hex_result);
    }

    #[test]
    fn should_test_failing_p2sh_tx_2() { // FIXME
        let db = get_test_database();
        let sats_per_byte = 23;
        let btc_network = BtcNetwork::Testnet;
        let btc_address = "mwi6VyZUqwqdu1DtQMruV4UzEqJADZzj6n"
            .to_string();
        let submission_material = get_sample_eos_submission_material_json_n(3);
        let action_proof = ActionProof::from_json(
            &submission_material
                .action_proofs[0]
        ).unwrap();
        let action_params = ActionParams::from_json(
            &submission_material
                .action_proofs[0]
                .action_json
                .data
        ).unwrap();

        let action_data = ActionData::new(action_proof, action_params);
        let redeem_params = parse_redeem_params_from_actions_data(
            &vec![action_data],
        ).unwrap();
        let utxo = get_sample_p2sh_utxo_and_value_2()
            .unwrap();
        save_utxos_to_db(&db, &vec![utxo])
            .unwrap();
        let pk = BtcPrivateKey::from_slice(
            &hex::decode(
        "2cc48e2f9066a0452e73cc7874f3fa8ba5ef705067d64bef627c686baa514336"
            ).unwrap(),
            btc_network.clone(),
        ).unwrap();
        pk
            .write_to_database(&db, &BTC_PRIVATE_KEY_DB_KEY.to_vec())
            .unwrap();
        put_btc_network_in_db(&db, &btc_network)
            .unwrap();
        put_btc_address_in_db(&db, &btc_address)
            .unwrap();
        let result = sign_txs_from_redeem_params(
            &db,
            sats_per_byte,
            btc_network,
            &redeem_params,
        ).unwrap();

        let result_hex = get_hex_tx_from_signed_btc_tx(&result);
        // From the actual failing tx!
        let expected_hex_result = "0100000001d8baf6344ab19575fe40ad81e5ca1fa57345025e40de3975f7d58c7266e7b7a6000000008f483045022100c2c86c3b03306bef189a862665bece4bdd01983f797b4f9e3edf6c5ba32ac243022038fdad04425cf175cdfcdeb5d7a783934b041f7a42c7b3077652a682a8352a9a014520396e1af63755ae83fecff1975c8addcce346c58cc98103c2781c527ae03915d1752102f55e923c43236f553424b863b1d589b9b67add4d8c49aeca7e667c4772e7b942acffffffff02b3150000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88ac33930000000000001976a914b19d7011a6107944209d5ebf9ef31a0fdf3115f188ac00000000";
        assert_eq!(result_hex, expected_hex_result);
    }

    #[test]
    fn should_x() { // FIXME
        simple_logger::init().unwrap();
        let db = get_test_database();
        let sats_per_byte = 23;
        let btc_network = BtcNetwork::Testnet;
        let btc_address = "mwi6VyZUqwqdu1DtQMruV4UzEqJADZzj6n"
            .to_string();
        let submission_material = get_sample_eos_submission_material_json_n(4);
        let action_proof = ActionProof::from_json(
            &submission_material
                .action_proofs[0]
        ).unwrap();
        let action_params = ActionParams::from_json(
            &submission_material
                .action_proofs[0]
                .action_json
                .data
        ).unwrap();
        let action_data = ActionData::new(action_proof, action_params);
        let redeem_params = parse_redeem_params_from_actions_data(
            &vec![action_data],
        ).unwrap();
        let utxo = get_sample_p2sh_utxo_and_value_3()
            .unwrap();
        save_utxos_to_db(&db, &vec![utxo])
            .unwrap();
        let pk = BtcPrivateKey::from_slice(
            &hex::decode(
        "040cc91d329860197e118a1ea26b7ed7042de8f991d0600df9e482c367bb1c45"
            ).unwrap(),
            btc_network.clone(),
        ).unwrap();
        pk
            .write_to_database(&db, &BTC_PRIVATE_KEY_DB_KEY.to_vec())
            .unwrap();
        put_btc_network_in_db(&db, &btc_network)
            .unwrap();
        put_btc_address_in_db(&db, &btc_address)
            .unwrap();
        let result = sign_txs_from_redeem_params(
            &db,
            sats_per_byte,
            btc_network,
            &redeem_params,
        ).unwrap();
        trace!("✔ Signed transaction: {:?}", result);

        let result_hex = get_hex_tx_from_signed_btc_tx(&result);
        // NOTE: From the actual failing tx! How to get this output?
        let incorrect_hex_result = "0100000001d8baf6344ab19575fe40ad81e5ca1fa57345025e40de3975f7d58c7266e7b7a6000000008f483045022100c2c86c3b03306bef189a862665bece4bdd01983f797b4f9e3edf6c5ba32ac243022038fdad04425cf175cdfcdeb5d7a783934b041f7a42c7b3077652a682a8352a9a014520396e1af63755ae83fecff1975c8addcce346c58cc98103c2781c527ae03915d1752102f55e923c43236f553424b863b1d589b9b67add4d8c49aeca7e667c4772e7b942acffffffff02b3150000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88ac33930000000000001976a914b19d7011a6107944209d5ebf9ef31a0fdf3115f188ac00000000";
        let correct_hex_result = "0100000001d8baf6344ab19575fe40ad81e5ca1fa57345025e40de3975f7d58c7266e7b7a6000000008f48304502210088a40269c4fe59aa8b2d6d0c5eb24f4e5d54879042806ea9140d00f7b4f372f4022077fa2da7c5fd70537d2a2006535f2151e3c9dbe1274664f2c59ba22cacca3c7a014520d11539e07a521c78c20381c98cc546e3ccdd8a5c97f1cffe83ae5537f61a6e39752102f55e923c43236f553424b863b1d589b9b67add4d8c49aeca7e667c4772e7b942acffffffff02b3150000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88ac469c0000000000001976a914b19d7011a6107944209d5ebf9ef31a0fdf3115f188ac00000000";
        // CONFIRMED THAT IT'S NOT THE ENDIANES!?? Maybe Not??
        //assert_eq!(result_hex, expected_hex_result);
        assert_eq!(result_hex, correct_hex_result);
    }

    #[test]
    fn should_y() { // FIXME
        simple_logger::init().unwrap();
        let db = get_test_database();
        let sats_per_byte = 23;
        let btc_network = BtcNetwork::Testnet;
        let btc_address = "mwi6VyZUqwqdu1DtQMruV4UzEqJADZzj6n"
            .to_string();
        let submission_material = get_sample_eos_submission_material_json_n(5);
        let action_proof = ActionProof::from_json(
            &submission_material
                .action_proofs[0]
        ).unwrap();
        let action_params = ActionParams::from_json(
            &submission_material
                .action_proofs[0]
                .action_json
                .data
        ).unwrap();
        let action_data = ActionData::new(action_proof, action_params);
        let redeem_params = parse_redeem_params_from_actions_data(
            &vec![action_data],
        ).unwrap();
        let utxo = get_sample_p2sh_utxo_and_value_4()
            .unwrap();
        save_utxos_to_db(&db, &vec![utxo])
            .unwrap();
        let pk = BtcPrivateKey::from_slice(
            &hex::decode(
        "44c5c5785546250ff72c647f9e0ef8ed4dc531ef908b53516723a52394e11de6"
            ).unwrap(),
            btc_network.clone(),
        ).unwrap();
        pk
            .write_to_database(&db, &BTC_PRIVATE_KEY_DB_KEY.to_vec())
            .unwrap();
        put_btc_network_in_db(&db, &btc_network)
            .unwrap();
        put_btc_address_in_db(&db, &btc_address)
            .unwrap();
        let result = sign_txs_from_redeem_params(
            &db,
            sats_per_byte,
            btc_network,
            &redeem_params,
        ).unwrap();
        trace!("✔ Signed transaction: {:?}", result);

        let result_hex = get_hex_tx_from_signed_btc_tx(&result);
        // NOTE: From the actual failing tx! How to get this output?
        let non_broadcastable_hex_result = "01000000012ae4f0fbd274c83895ee0d1fc3860506014c6971534c471a54acbee1be135216000000008e47304402207147dddcef10f364d37c587455d54b5ecebc6fd4fb3633e371f63e058b765c7e022047a32f1c3fb9af81e99ef44a6f549519c0d011b1fc4a0ffebf92e5f68465fda901452012b313ab41331418c085971cfdbe4d79ddc9fb49edce6bb554b63133252122bf752103e0a77075d064f5064a45017a2f3aa1ccca775aaed4d7eabe344e651ea29e4031acffffffff02f7130000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88acef940000000000001976a91470c1c80d3129469426f1c9bba8bf97f2edae5ef588ac00000000";
        // and this one has theh add/nonce has the INCORRECT way around 12b3131...
        let broadcastable_hex_result = "01000000012ae4f0fbd274c83895ee0d1fc3860506014c6971534c471a54acbee1be135216000000008e473044022036f99ecf1882241c1163587a0b906cb2becad15ba3440c36d26c9a7151d2ff5b02204469cbe8e0030e60a4cca732dfbc5e5b57f4c47b228d33053c751126977e8f2a014520bf2221253331b654b56bceed49fbc9dd794dbefd1c9785c018143341ab13b312752103e0a77075d064f5064a45017a2f3aa1ccca775aaed4d7eabe344e651ea29e4031acffffffff02f7130000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88ac029e0000000000001976a914b19d7011a6107944209d5ebf9ef31a0fdf3115f188ac00000000";
        // So this has it the add/nonce hash the correct way around per the repl. bf222...
        //
        let maybe_this_one = "01000000012ae4f0fbd274c83895ee0d1fc3860506014c6971534c471a54acbee1be135216000000008e47304402205061dc44c5fab8a9c409b4fd85f899d8d8d1bcdc6c4511643dcd100c53b1a2f2022005434f005098a975d51ece254ad8008cb6619888f75cced69a4845e9db0cee1b014520bf2221253331b654b56bceed49fbc9dd794dbefd1c9785c018143341ab13b312752103e0a77075d064f5064a45017a2f3aa1ccca775aaed4d7eabe344e651ea29e4031acffffffff02f7130000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88acef940000000000001976a91470c1c80d3129469426f1c9bba8bf97f2edae5ef588ac00000000";
        //assert_eq!(result_hex, broadcastable_hex_result);
        assert_eq!(result_hex, maybe_this_one);
        //assert_eq!(result_hex, non_broadcastable_result);
    }
    */
}
