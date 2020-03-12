use std::str::FromStr;
use eos_primitives::{
    Checksum256,
    Action as EosAction,
    AccountName as EosAccountName,
    BlockHeader as EosBlockHeader,
    ActionReceipt as EosActionReceipt,
};
use crate::btc_on_eos::{
    utils::convert_hex_to_checksum256,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_crypto::eos_signature::EosSignature,
        parse_eos_actions::parse_eos_action_json,
    },
};

pub type EosAmount = String;
pub type EosAddress = String;
pub type MerkleProof = Vec<String>;
pub type EosAddresses = Vec<String>;
pub type EosAmounts = Vec<EosAmount>;
pub type ActionProofs = Vec<ActionProof>;
pub type MerkleProofs = Vec<MerkleProof>;
pub type EosSignatures = Vec<EosSignature>;
pub type ActionsParams = Vec<ActionParams>;
pub type ActionProofJsons = Vec<ActionProofJson>;
pub type Sha256HashedMessage = secp256k1::Message;
pub type ActionParamsJsons = Vec<ActionParamsJson>;
pub type AuthorizationJsons = Vec<AuthorizationJson>;
pub type EosSignedTransactions= Vec<EosSignedTransaction>;

#[derive(Debug, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub struct ActionParams {
    pub memo: String,
    pub quantity: u64,
    pub sender: EosAccountName,
}

impl ActionParams {
    pub fn from_json(json: &ActionParamsJson) -> Result<Self> {
        Ok(
            ActionParams {
                memo: json.memo.clone(),
                sender: EosAccountName::from_str(&json.sender)?,
                quantity: json
                    .quantity
                    .split_whitespace()
                    .collect::<Vec<&str>>()[0]
                    .parse::<u64>()?,
            }
        )
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum EosNetwork {
    Mainnet,
    Testnet,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct EosSignedTransaction {
    pub nonce: usize,
    pub amount: String,
    pub recipient: String,
    pub signature: String,
    pub transaction: String,
}

impl EosSignedTransaction {
    pub fn new(
        signature: String,
        transaction: String,
        nonce: usize,
        recipient: String,
        amount: String,
    ) -> EosSignedTransaction {
        EosSignedTransaction {
            signature,
            transaction,
            nonce,
            amount,
            recipient,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterial {
    pub action_proofs: ActionProofs,
    pub action_params: ActionsParams,
    pub block_header: EosBlockHeader,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterialJson {
    pub action_proofs: ActionProofJsons,
    pub block_header: EosBlockHeaderJson,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionParamsJson {
    pub memo: String,
    pub sender: String,
    pub quantity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosBlockHeaderJson {
    pub confirmed: u16,
    pub producer: String,
    pub previous: String,
    pub block_id: String,
    pub block_num: usize,
    pub timestamp: String,
    pub action_mroot: String,
    pub schedule_version: u32,
    pub transaction_mroot: String,
    pub producer_signature: String,
    pub header_extension: Option<Vec<String>>,
    pub new_producers: Option<ProducerScheduleJson>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProducerScheduleJson {
    pub version: u32,
    pub producers: Vec<ProducerKeyJson>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProducerKeyJson {
    pub producer_name: String, // To become AccountName
    pub block_signing_key: String, // To become public key
}

#[derive(Debug)]
pub struct EosRawTxData {
    pub sender: String,
    pub mint_nonce: u64,
    pub receiver: String,
    pub asset_amount: u64,
    pub asset_name: String,
    pub eth_address: String,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionProof {
    pub action: EosAction,
    pub tx_id: Checksum256,
    pub action_index: usize,
    pub block_id: Checksum256,
    pub serialized_action: Bytes,
    pub action_proof: MerkleProof,
    pub action_digest: Checksum256,
    pub action_receipt_digest: Bytes,
    pub serialized_action_receipt: Bytes,
}

impl ActionProof {
    pub fn from_json(json: &ActionProofJson) -> Result<Self> {
        Ok(
            ActionProof {
                action_index:
                    json.action_index.clone(),
                action_proof:
                    json.action_proof.clone(),
                serialized_action:
                    hex::decode(&json.serialized_action)?,
                tx_id:
                    convert_hex_to_checksum256(&json.tx_id)?,
                action_receipt_digest:
                    hex::decode(&json.action_receipt_digest)?,
                action:
                    parse_eos_action_json(&json.action_json)?,
                block_id:
                    convert_hex_to_checksum256(&json.block_id)?,
                serialized_action_receipt:
                    hex::decode(&json.serialized_action_receipt)?,
                action_digest:
                    convert_hex_to_checksum256(&json.action_digest)?,
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosActionJson {
    pub name: String,
    pub account: String,
    pub data: ActionParamsJson,
    pub hex_data: Option<String>,
    pub authorization: AuthorizationJsons,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorizationJson {
    pub actor: String,
    pub permission: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionProofJson {
    pub tx_id: String,
    pub block_id: String,
    pub action_index: usize,
    pub action_digest: String,
    pub action_proof: MerkleProof,
    pub serialized_action: String,
    pub action_json: EosActionJson,
    pub action_receipt_digest: String,
    pub serialized_action_receipt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessedTxIds(pub Vec<String>);

impl ProcessedTxIds {
    pub fn init() -> Self {
        ProcessedTxIds(vec![])
    }

    pub fn add(mut self, tx_id: String) -> Result<Self> {
        if !Self::contains(&self, &tx_id) {
            self.0.push(tx_id);
        }
        Ok(self)
    }

    pub fn contains(&self, tx_id: &String) -> bool {
        self.0.contains(tx_id)
    }
}
