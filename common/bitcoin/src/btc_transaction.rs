use common::types::{Bytes, Result};

use crate::{
    bitcoin_crate_alias::{
        blockdata::transaction::{Transaction as BtcTransaction, TxIn as BtcUtxo},
        Sighash,
    },
    btc_constants::{BTC_TX_LOCK_TIME, BTC_TX_VERSION, DUST_AMOUNT},
    btc_recipients_and_amounts::BtcRecipientsAndAmounts,
    btc_utils::{
        create_new_pay_to_pub_key_hash_output,
        get_p2sh_redeem_script_sig,
        get_p2sh_script_sig_from_redeem_script,
        get_script_sig,
    },
    deposit_address_info::DepositAddressInfo,
    utxo_manager::BtcUtxosAndValues,
    BtcPrivateKey,
};

pub const SIGN_ALL_HASH_TYPE: u8 = 1;

pub fn create_signed_raw_btc_tx_for_n_input_n_outputs(
    sats_per_byte: u64,
    recipient_addresses_and_amounts: BtcRecipientsAndAmounts,
    remainder_btc_address: &str,
    btc_private_key: &BtcPrivateKey,
    utxos_and_values: BtcUtxosAndValues,
) -> Result<BtcTransaction> {
    let inputs = utxos_and_values.to_utxos()?;
    if inputs.is_empty() {
        return Err("Cannot create BTC transaction with zero inputs!".into());
    };
    let mut zero_change_outputs = recipient_addresses_and_amounts.to_tx_outputs();
    let total_to_spend: u64 = recipient_addresses_and_amounts.sum();
    // NOTE: There will likely be a change output, which we need here in order to get correct tx size.
    // If there's no change output, we'll just end up paying a slightly higher fee.
    let zero_change_output = create_new_pay_to_pub_key_hash_output(0, remainder_btc_address)?;
    zero_change_outputs.push(zero_change_output);
    let zero_change_tx = BtcTransaction {
        input: inputs.clone(),
        output: zero_change_outputs,
        version: BTC_TX_VERSION,
        lock_time: BTC_TX_LOCK_TIME,
    };
    let fee = zero_change_tx.size() as u64 * sats_per_byte;
    let utxo_total = utxos_and_values.sum();
    info!("✔ UTXO(s) total:  {}", utxo_total);
    info!("✔ Outgoing total: {}", total_to_spend);
    info!("✔ Tx fee:         {}", fee);
    if total_to_spend + fee > utxo_total {
        return Err("✘ Not enough UTXO value to make transaction!".into());
    };
    info!("✔ Change amount:  {}", utxo_total - (total_to_spend + fee));
    let change_amount = utxo_total - total_to_spend - fee;
    let mut outputs = recipient_addresses_and_amounts.to_tx_outputs();
    if change_amount > 0 {
        if change_amount <= *DUST_AMOUNT {
            // NOTE: Dust is taken into account when getting UTXOs. This is just here as another
            // line of defense against accidentally making dust outputs.
            return Err(format!(
                "Cannot create BTC transaction, change output is {} satoshis, which is dust!",
                change_amount
            )
            .into());
        };
        outputs.push(create_new_pay_to_pub_key_hash_output(
            change_amount,
            remainder_btc_address,
        )?)
    };
    let tx = BtcTransaction {
        input: inputs,
        output: outputs,
        version: BTC_TX_VERSION,
        lock_time: BTC_TX_LOCK_TIME,
    };
    let signatures = utxos_and_values
        .iter()
        .map(|utxo_and_value| utxo_and_value.get_utxo())
        .enumerate()
        .map(|(i, utxo)| {
            let script = match utxos_and_values[i].clone().maybe_deposit_info_json {
                None => {
                    info!("✔ Signing a `p2pkh` UTXO!");
                    utxo?.script_sig
                },
                Some(deposit_info_json) => {
                    info!("✔ Signing a `p2sh` UTXO!");
                    get_p2sh_redeem_script_sig(
                        &btc_private_key.to_public_key_slice(),
                        &DepositAddressInfo::from_json(&deposit_info_json)?.commitment_hash,
                    )
                },
            };
            Ok(tx.signature_hash(i, &script, SIGN_ALL_HASH_TYPE as u32))
        })
        .map(|hash: Result<Sighash>| Ok(hash?.to_vec()))
        .map(|tx_hash_to_sign: Result<Bytes>| {
            btc_private_key.sign_hash_and_append_btc_hash_type(tx_hash_to_sign?.to_vec(), SIGN_ALL_HASH_TYPE)
        })
        .collect::<Result<Vec<Bytes>>>()?;

    let utxos_with_signatures = utxos_and_values
        .iter()
        .map(|utxo_and_value| utxo_and_value.get_utxo())
        .enumerate()
        .map(|(i, maybe_utxo)| {
            let utxo = maybe_utxo?;
            let script_sig = match utxos_and_values[i].clone().maybe_deposit_info_json {
                None => {
                    info!("✔ Spending a `p2pkh` UTXO!");
                    get_script_sig(&signatures[i], &btc_private_key.to_public_key_slice())
                },
                Some(deposit_info_json) => {
                    info!("✔ Spending a `p2sh` UTXO!");
                    get_p2sh_script_sig_from_redeem_script(
                        &signatures[i],
                        &get_p2sh_redeem_script_sig(
                            &btc_private_key.to_public_key_slice(),
                            &DepositAddressInfo::from_json(&deposit_info_json)?.commitment_hash,
                        ),
                    )
                },
            };
            Ok(BtcUtxo {
                script_sig,
                sequence: utxo.sequence,
                witness: utxo.witness.clone(),
                previous_output: utxo.previous_output,
            })
        })
        .collect::<Result<Vec<BtcUtxo>>>()?;
    Ok(BtcTransaction {
        output: tx.output,
        version: tx.version,
        lock_time: tx.lock_time,
        input: utxos_with_signatures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        btc_recipients_and_amounts::BtcRecipientAndAmount,
        btc_utils::{get_hex_tx_from_signed_btc_tx, get_tx_id_from_signed_btc_tx},
        test_utils::{
            get_sample_btc_private_key,
            get_sample_p2pkh_utxo_and_value,
            get_sample_p2pkh_utxo_and_value_n,
            SAMPLE_TARGET_BTC_ADDRESS,
        },
    };

    #[test]
    fn should_serialize_1_input_1_output_tx_correctly() {
        let expected_tx_id = "655ce4e0b4b5e1617bf44e997c760e1479fc4db129ab5f3c102f7f18c156f66a";
        let expected_serialized_tx = "01000000010e8d588f88d5624148070a8cd79508da8cb76625e4fcdb19a5fc996aa843bf04000000006a47304402203e1c206d4812d09831d171c225beea3aa119f13fb5cbaf5bc56abb31d92127ac02202abb509818daf3dc8839a898030cd6d05eb9fe500f87369a15e67ef6161e5ef1012103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7ffffffff0233023300000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88ac67040000000000001976a91454102783c8640c5144d039cea53eb7dbb470081488ac00000000";
        let sats_per_byte = 23;
        let recipient_addresses_and_amounts = BtcRecipientsAndAmounts::new(vec![BtcRecipientAndAmount::new(
            "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM",
            3342899,
        )
        .unwrap()]);
        let btc_private_key = get_sample_btc_private_key();
        let remainder_btc_address = SAMPLE_TARGET_BTC_ADDRESS;
        let utxos_and_values = BtcUtxosAndValues::new(vec![get_sample_p2pkh_utxo_and_value()]);
        let final_signed_tx = create_signed_raw_btc_tx_for_n_input_n_outputs(
            sats_per_byte,
            recipient_addresses_and_amounts,
            remainder_btc_address,
            &btc_private_key,
            utxos_and_values,
        )
        .unwrap();
        let tx_id = get_tx_id_from_signed_btc_tx(&final_signed_tx);
        let result_hex = get_hex_tx_from_signed_btc_tx(&final_signed_tx);
        assert_eq!(result_hex, expected_serialized_tx);
        assert_eq!(tx_id, expected_tx_id);
    }

    #[test]
    fn should_serialize_1_input_2_outputs_tx_correctly() {
        let expected_tx_id = "2151543a306f9fe840c9c98049c9b7cc83ea0aabf0f18f1e9d3d7fd2a2da2ba8";
        let expected_serialized_tx = "0100000001b5f75f17e28fa0edaa8148bc6e255797975e1529d9ad97d790914f7c6be26bb5020000006a47304402205ef2b604530ffaa4c7f86ae9a96caefd4bb03747ee5faad0f545ce43360550f5022075a5a61873b13e8a311c82f2e044a63a771fd4383663069a7aff8bf6c97b04d9012103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7ffffffff0239050000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88ac76aa0e00000000001976a91454102783c8640c5144d039cea53eb7dbb470081488ac00000000";
        let utxos_and_values = BtcUtxosAndValues::new(vec![get_sample_p2pkh_utxo_and_value_n(2).unwrap()]);
        let sats_per_byte = 23;
        let recipient_addresses_and_amounts = BtcRecipientsAndAmounts::new(vec![BtcRecipientAndAmount::new(
            "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM",
            1337,
        )
        .unwrap()]);
        let remainder_btc_address = SAMPLE_TARGET_BTC_ADDRESS;
        let btc_private_key = get_sample_btc_private_key();
        let final_signed_tx = create_signed_raw_btc_tx_for_n_input_n_outputs(
            sats_per_byte,
            recipient_addresses_and_amounts,
            remainder_btc_address,
            &btc_private_key,
            utxos_and_values,
        )
        .unwrap();
        let tx_id = get_tx_id_from_signed_btc_tx(&final_signed_tx);
        let result_hex = get_hex_tx_from_signed_btc_tx(&final_signed_tx);
        assert_eq!(result_hex, expected_serialized_tx);
        assert_eq!(tx_id, expected_tx_id);
    }

    #[test]
    fn should_serialize_tx_with_n_inputs_and_n_outputs() {
        let expected_tx_id = "4b8c6951bd174bc39af84e001a997123f8b157f098b049d22212e536a6a84953";
        let expected_result = "0100000002637cb89f9647c2de31478d554696fb1878f86fd91e399989747e3c6ff296828f000000006a47304402202d53a8171ce4dc1ef7ed854640128053a3549993620a9bab1e99c4aeeb7d04c6022067741ffd0a95d2a69fd593c87909cb9025465343f0b72e7ba26e173a91b90bcf012103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7ffffffff637cb89f9647c2de31478d554696fb1878f86fd91e399989747e3c6ff296828f010000006b483045022100dc66b33210cbdfd0d3d43f2a844d8b3a7418a34b69b3863bcbc6582cc63b99710220333cfc21b26a01df955f92aba8c204c35da1d003e55262e9e3dc5d1737ac50b4012103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7ffffffff039a020000000000001976a9149ae6e42c56f1ea319cfc704ad50db0683015029b88ac39050000000000001976a91493f36f39571997887fb4eff72d7a96259c34292288ac7bc80e00000000001976a91454102783c8640c5144d039cea53eb7dbb470081488ac00000000";
        let utxos_and_values = BtcUtxosAndValues::new(vec![
            get_sample_p2pkh_utxo_and_value_n(3).unwrap(),
            get_sample_p2pkh_utxo_and_value_n(4).unwrap(),
        ]);
        let sats_per_byte = 23;
        let btc_private_key = get_sample_btc_private_key();
        let remainder_btc_address = SAMPLE_TARGET_BTC_ADDRESS;
        let recipient_addresses_and_amounts = BtcRecipientsAndAmounts::new(vec![
            BtcRecipientAndAmount::new("mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM", 666).unwrap(),
            BtcRecipientAndAmount::new("mu1FFNnoiMytR5tKGXp6M1XhUZFQd3Mc8n", 1337).unwrap(),
        ]);
        let final_signed_tx = create_signed_raw_btc_tx_for_n_input_n_outputs(
            sats_per_byte,
            recipient_addresses_and_amounts,
            remainder_btc_address,
            &btc_private_key,
            utxos_and_values,
        )
        .unwrap();
        let tx_id = get_tx_id_from_signed_btc_tx(&final_signed_tx);
        let result_hex = get_hex_tx_from_signed_btc_tx(&final_signed_tx);
        assert_eq!(result_hex, expected_result);
        assert_eq!(tx_id, expected_tx_id);
    }
}
