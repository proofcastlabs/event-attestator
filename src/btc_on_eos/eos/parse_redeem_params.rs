use std::str::from_utf8;
use eos_primitives::{
    Symbol as EosSymbol,
    AccountName as EosAccountName,
};
use crate::btc_on_eos::{
    traits::DatabaseInterface,
    utils::convert_bytes_to_u64,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_state::EosState,
        eos_types::{
            ActionData,
            ActionsData,
            RedeemParams,
        },
    },
};

#[allow(dead_code)] // TODO Use when checking for correct sybmol!
fn get_eos_symbol_from_action_data(
    action_data: &Bytes
) -> Result<EosSymbol> {
    Ok(
        EosSymbol::new(
            convert_bytes_to_u64(&action_data[16..24].to_vec())?
        )
    )
}

fn get_eos_amount_from_action_data(
    action_data: &Bytes
) -> Result<u64> {
    convert_bytes_to_u64(&action_data[8..16].to_vec())
}

fn get_redeem_action_sender_from_action_data(
    action_data: &Bytes
) -> Result<EosAccountName> {
    Ok(
        EosAccountName::new(
            convert_bytes_to_u64(&action_data[..8].to_vec())?
        )
    )
}

fn get_redeem_address_from_action_data(
    action_data: &Bytes,
) -> Result<String> {
    Ok(from_utf8(&action_data[25..])?.to_string())
}

impl RedeemParams {
    pub fn from_action_data(
        action_data: &ActionData,
    ) -> Result<Self> {
        Ok(
            RedeemParams {
                amount: get_eos_amount_from_action_data(
                    &action_data.action_proof.action.data,
                )?,
                from: get_redeem_action_sender_from_action_data(
                    &action_data.action_proof.action.data,
                )?,
                recipient: get_redeem_address_from_action_data(
                    &action_data.action_proof.action.data,
                )?,
                originating_tx_id: action_data.action_proof.tx_id.clone(),
            }
        )
    }
}

pub fn parse_redeem_params_from_actions_data(
    actions_data: &ActionsData
) -> Result<Vec<RedeemParams>> {
    actions_data
        .iter()
        .map(|action_data| RedeemParams::from_action_data(action_data))
        .collect()
}

pub fn maybe_parse_redeem_params_and_put_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing redeem params from actions data...");
    parse_redeem_params_from_actions_data(&state.actions_data)
        .and_then(|params| {
            debug!("✔ Parsed {} sets of params!", params.len());
            state.add_redeem_params(params)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::btc_on_eos::{
        utils::convert_hex_to_checksum256,
        eos::eos_test_utils::get_sample_eos_submission_material_n,
    };

    #[test]
    fn should_get_sender_from_action_data() {
        let expected_result = EosAccountName::from_str("provtestable")
            .unwrap();
        let action_data = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .action
            .data
            .clone();
        let result = get_redeem_action_sender_from_action_data(
            &action_data
        ).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_symbol_from_action_data() {
        let expected_result = EosSymbol::from_str("8,PFFF")
            .unwrap();
        let action_data = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .action
            .data
            .clone();
        let result = get_eos_symbol_from_action_data(&action_data)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_amount_from_action_data() {
        let expected_result: u64 = 5111;
        let action_data = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .action
            .data
            .clone();
        let result = get_eos_amount_from_action_data(&action_data)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_redeem_address_serialized_action() {
        let expected_result = "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM"
            .to_string();
        let action_data = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .action_proof
            .action
            .data
            .clone();
        let result = get_redeem_address_from_action_data(
            &action_data
        ).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_redeem_params_from_action_data_2() {
        let expected_result = RedeemParams {
            amount: 1,
            recipient: "mr6ioeUxNMoavbr2VjaSbPAovzzgDT7Su9"
                .to_string(),
            from: EosAccountName::from_str("provabletest")
                .unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
            &"34dff748d2bbb9504057d4be24c69b8ac38b2905f7e911dd0e9ed3bf369bae03"
                .to_string()
            ).unwrap(),
        };
        let action_data = get_sample_eos_submission_material_n(2)
            .actions_data[0]
            .clone();
        let result = RedeemParams::from_action_data(&action_data)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_redeem_params_from_action_data_3() {
        let expected_result = RedeemParams {
            amount: 5314,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM"
                .to_string(),
            from: EosAccountName::from_str("provtestable")
                .unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
            &"084129e4cae2b8fb2c51fd6e569ac0f80db4de5f15ff3c2776233d9ae42aad64"
                .to_string()
            ).unwrap(),
        };
        let action_data = get_sample_eos_submission_material_n(3)
            .actions_data[0]
            .clone();
        let result = RedeemParams::from_action_data(&action_data)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_redeem_params_from_action_data_4() {
        let expected_result = RedeemParams {
            amount: 5555,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM"
                .to_string(),
            from: EosAccountName::from_str("provtestable")
                .unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
            &"8eaafcb796002a12e0f48ebc0f832bacca72a8b370e00967c65619a2c1814a04"
                .to_string()
            ).unwrap(),
        };
        let action_data = get_sample_eos_submission_material_n(4)
            .actions_data[0]
            .clone();
        let result = RedeemParams::from_action_data(&action_data)
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_redeem_params_from_action_data_5() {
        let expected_result = RedeemParams {
            amount: 5111,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM"
                .to_string(),
            from: EosAccountName::from_str("provtestable")
                .unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
            &"aebe7cd1a4687485bc5db87bfb1bdfb44bd1b7f9c080e5cb178a411fd99d2fd5"
                .to_string()
            ).unwrap(),
        };
        let action_data = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .clone();
        let result = RedeemParams::from_action_data(&action_data)
            .unwrap();
        assert_eq!(result, expected_result);
    }
}
