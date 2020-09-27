pub use eos_primitives::Checksum256;
use eos_primitives::{
    Action as EosAction,
    ActionReceipt as EosActionReceipt,
};
use crate::{
    types::Result,
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

