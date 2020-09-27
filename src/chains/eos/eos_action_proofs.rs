use std::str::from_utf8;
use ethereum_types::{
    U256,
    Address as EthAddress,
};
use eos_primitives::{
    Checksum256,
    Action as EosAction,
    Symbol as EosSymbol,
    AccountName as EosAccountName,
    ActionReceipt as EosActionReceipt,
};
use crate::{
    types::Result,
    utils::convert_bytes_to_u64,
    btc_on_eos::eos::redeem_info::BtcOnEosRedeemInfo,
    erc20_on_eos::eos::redeem_info::Erc20OnEosRedeemInfo,
    chains::eos::{
        eos_types:: MerkleProof,
        eos_utils::convert_hex_to_checksum256,
        parse_eos_actions::parse_eos_action_json,
        parse_eos_action_receipts::parse_eos_action_receipt_json,
    },
};

pub type EosActionProofs = Vec<EosActionProof>;
pub type EosActionProofJsons = Vec<EosActionProofJson>;
pub type AuthSequenceJsons = Vec<AuthSequenceJson>;
pub type AuthorizationJsons = Vec<AuthorizationJson>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EosActionProof {
    pub action: EosAction,
    pub tx_id: Checksum256,
    pub action_proof: MerkleProof,
    pub action_receipt: EosActionReceipt,
}

impl EosActionProof {
    #[allow(dead_code)] // TODO Use when checking for correct symbol!
    fn get_eos_symbol(&self) -> Result<EosSymbol> {
        Ok(EosSymbol::new(convert_bytes_to_u64(&self.action.data[16..24].to_vec())?))
    }

    fn get_btc_on_eos_eos_amount(&self) -> Result<u64> {
        convert_bytes_to_u64(&self.action.data[8..16].to_vec())
    }

    fn get_erc20_on_eos_btc_redeem_amount(&self) -> Result<U256> {
        Ok(U256::from(&self.action.data[8..16])) // FIXME TODO Test this!
    }

    fn get_redeem_action_sender(&self) -> Result<EosAccountName> {
        Ok(EosAccountName::new(convert_bytes_to_u64(&self.action.data[..8].to_vec())?))
    }

    fn get_btc_on_eos_btc_redeem_address(&self) -> Result<String> {
        Ok(from_utf8(&self.action.data[25..])?.to_string())
    }

    fn get_erc20_on_eos_btc_redeem_address(&self) -> Result<EthAddress> {
        Ok(EthAddress::from_slice(&self.action.data[25..])) // FIXME / TODO Test this!
    }

    pub fn from_json(json: &EosActionProofJson) -> Result<Self> {
        Ok(
            EosActionProof {
                action_proof: json.action_proof.clone(),
                tx_id: convert_hex_to_checksum256(&json.tx_id)?,
                action: parse_eos_action_json(&json.action_json)?,
                action_receipt: parse_eos_action_receipt_json(&json.action_receipt_json)?,
            }
        )
    }

    pub fn to_btc_on_eos_redeem_info(&self) -> Result<BtcOnEosRedeemInfo> {
        Ok(BtcOnEosRedeemInfo {
            originating_tx_id: self.tx_id,
            from: self.get_redeem_action_sender()?,
            amount: self.get_btc_on_eos_eos_amount()?,
            recipient: self.get_btc_on_eos_btc_redeem_address()?,
            global_sequence: self.action_receipt.global_sequence,
        })
    }

    pub fn to_erc20_on_eos_redeem_info(&self) -> Result<Erc20OnEosRedeemInfo> {
        Ok(Erc20OnEosRedeemInfo {
            originating_tx_id: self.tx_id,
            from: self.get_redeem_action_sender()?,
            amount: self.get_erc20_on_eos_btc_redeem_amount()?,
            global_sequence: self.action_receipt.global_sequence,
            recipient: self.get_erc20_on_eos_btc_redeem_address()?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosActionProofJson {
    pub tx_id: String,
    pub block_id: String,
    pub action_index: usize,
    pub action_digest: String,
    pub action_proof: MerkleProof,
    pub serialized_action: String,
    pub action_json: EosActionJson,
    pub action_receipt_digest: String,
    pub serialized_action_receipt: String,
    pub action_receipt_json: EosActionReceiptJson,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosActionReceiptJson {
    pub receiver: String,
    pub act_digest: String,
    pub global_sequence: u64,
    pub recv_sequence:  u64,
    pub auth_sequence: AuthSequenceJsons,
    pub code_sequence: usize,
    pub abi_sequence: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthSequenceJson(pub String, pub u64);


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosActionJson {
    pub name: String,
    pub account: String,
    pub hex_data: Option<String>,
    pub authorization: AuthorizationJsons,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizationJson {
    pub actor: String,
    pub permission: String,
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::btc_on_eos::eos::eos_test_utils::get_sample_eos_submission_material_n;

    #[test]
    fn should_get_sender() {
        let expected_result = EosAccountName::from_str("provtestable")
            .unwrap();
        let result = get_sample_eos_submission_material_n(1)
            .action_proofs[0]
            .get_redeem_action_sender()
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_symbol() {
        let expected_result = EosSymbol::from_str("8,PFFF")
            .unwrap();
        let result = get_sample_eos_submission_material_n(1)
            .action_proofs[0]
            .get_eos_symbol()
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_amount() {
        let expected_result: u64 = 5111;
        let result = get_sample_eos_submission_material_n(1)
            .action_proofs[0]
            .get_btc_on_eos_eos_amount()
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_btc_redeem_address_serialized_action() {
        let expected_result = "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM"
            .to_string();
        let result = get_sample_eos_submission_material_n(1)
            .action_proofs[0]
            .get_btc_on_eos_btc_redeem_address()
            .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_redeem_infos_from_action_proof_2() {
        let expected_result = BtcOnEosRedeemInfo {
            global_sequence: 577606126,
            amount: 1,
            recipient: "mr6ioeUxNMoavbr2VjaSbPAovzzgDT7Su9".to_string(),
            from: EosAccountName::from_str("provabletest").unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
                &"34dff748d2bbb9504057d4be24c69b8ac38b2905f7e911dd0e9ed3bf369bae03".to_string()
            ).unwrap(),
        };
        let action_proof = get_sample_eos_submission_material_n(2).action_proofs[0].clone();
        let result = action_proof.to_btc_on_eos_redeem_info().unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_redeem_infos_from_action_proof_3() {
        let expected_result = BtcOnEosRedeemInfo {
            global_sequence: 583774614,
            amount: 5666,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
            from: EosAccountName::from_str("provabletest").unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
                &"51f0dbbaf6989e9b980d0fa18bd70ddfc543851ff65140623d2cababce2ceb8c".to_string()
            ).unwrap(),
        };
        let action_proof = get_sample_eos_submission_material_n(3).action_proofs[0].clone();
        let result = action_proof.to_btc_on_eos_redeem_info().unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_redeem_infos_from_action_proof_4() {
        let expected_result = BtcOnEosRedeemInfo {
            global_sequence: 579818529,
            amount: 5555,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
            from: EosAccountName::from_str("provtestable").unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
                &"8eaafcb796002a12e0f48ebc0f832bacca72a8b370e00967c65619a2c1814a04".to_string()
            ).unwrap(),
        };
        let action_proof = get_sample_eos_submission_material_n(4).action_proofs[0].clone();
        let result = action_proof.to_btc_on_eos_redeem_info().unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_btc_on_eos_redeem_infos_from_action_proof_5() {
        let expected_result = BtcOnEosRedeemInfo {
            global_sequence: 579838915,
            amount: 5111,
            recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
            from: EosAccountName::from_str("provtestable").unwrap(),
            originating_tx_id: convert_hex_to_checksum256(
                &"aebe7cd1a4687485bc5db87bfb1bdfb44bd1b7f9c080e5cb178a411fd99d2fd5".to_string()
            ).unwrap(),
        };
        let action_proof = get_sample_eos_submission_material_n(1).action_proofs[0].clone();
        let result = action_proof.to_btc_on_eos_redeem_info().unwrap();
        assert_eq!(result, expected_result);
    }
}
