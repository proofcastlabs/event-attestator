use crate::{
    chains::eos::{
        eos_global_sequences::GlobalSequence,
        eos_types::MerkleProof,
        eos_utils::convert_hex_to_checksum256,
        parse_eos_action_receipts::parse_eos_action_receipt_json,
    },
    types::{Bytes, Result},
    utils::convert_bytes_to_u64,
};
use eos_primitives::{
    AccountName as EosAccountName,
    AccountName,
    Action as EosAction,
    ActionName,
    ActionReceipt as EosActionReceipt,
    Checksum256,
    PermissionLevel,
    PermissionLevels,
    SerializeData,
};
use std::str::FromStr;

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
    pub fn get_global_sequence(&self) -> GlobalSequence {
        self.action_receipt.global_sequence
    }

    pub fn get_serialized_action(&self) -> Bytes {
        self.action.to_serialize_data()
    }

    pub fn get_action_sender(&self) -> Result<EosAccountName> {
        let account_name = EosAccountName::new(convert_bytes_to_u64(&self.action.data[..8].to_vec())?);
        debug!("✔ Account name parsed from redeem action: {}", account_name);
        Ok(account_name)
    }

    pub fn from_json(json: &EosActionProofJson) -> Result<Self> {
        Ok(EosActionProof {
            action: json.action_json.to_action()?,
            action_proof: json.action_proof.clone(),
            tx_id: convert_hex_to_checksum256(&json.tx_id)?,
            action_receipt: parse_eos_action_receipt_json(&json.action_receipt_json)?,
        })
    }

    pub fn get_action_eos_account(&self) -> EosAccountName {
        self.action.account
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
    pub recv_sequence: u64,
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

impl EosActionJson {
    fn parse_authorization_json(authorization_json: &AuthorizationJson) -> Result<PermissionLevel> {
        Ok(PermissionLevel::from_str(
            authorization_json.actor.clone(),
            authorization_json.permission.clone(),
        )?)
    }

    fn parse_authorization_jsons(authorization_jsons: &[AuthorizationJson]) -> Result<PermissionLevels> {
        authorization_jsons.iter().map(Self::parse_authorization_json).collect()
    }

    fn deserialize_action_data(maybe_hex_data: &Option<String>) -> Result<Bytes> {
        match maybe_hex_data {
            Some(string) => Ok(hex::decode(string)?),
            None => Err("✘ Failed to decode hex_data field of action!".into()),
        }
    }

    pub fn to_action(&self) -> Result<EosAction> {
        Ok(EosAction {
            name: ActionName::from_str(&self.name)?,
            account: AccountName::from_str(&self.account)?,
            data: Self::deserialize_action_data(&self.hex_data)?,
            authorization: Self::parse_authorization_jsons(&self.authorization)?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizationJson {
    pub actor: String,
    pub permission: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eos::eos_test_utils::{
        get_sample_eos_submission_material_json_n,
        get_sample_eos_submission_material_n,
    };
    use std::str::FromStr;

    fn get_sample_action_proof() -> EosActionProof {
        get_sample_eos_submission_material_n(1).action_proofs[0].clone()
    }

    #[test]
    fn should_get_sender() {
        let proof = get_sample_action_proof();
        let expected_result = EosAccountName::from_str("provtestable").unwrap();
        let result = proof.get_action_sender().unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_global_sequence_from_proof() {
        let proof = get_sample_action_proof();
        let result = proof.get_global_sequence();
        let expected_result = 579838915;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_action_eos_account_from_proof() {
        let proof = get_sample_action_proof();
        let result = proof.get_action_eos_account();
        let expected_result = EosAccountName::from_str("pbtctokenxxx").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_serialized_action_from_proof() {
        let proof = get_sample_action_proof();
        let result = proof.get_serialized_action();
        let expected_result = "d07b9f0ad28cf2a90000000048a592ba01a0e23119abbce9ad00000000a8ed32323ba0e23119abbce9adf7130000000000000850464646000000226d75647a7843713961435134556e61394d6d6179764a56434631546a39667970694d";
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_action_proof_from_json() {
        let json = get_sample_eos_submission_material_json_n(1).action_proofs[0].clone();
        let result = EosActionProof::from_json(&json);
        assert!(result.is_ok());
    }
}
