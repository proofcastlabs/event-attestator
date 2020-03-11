use eos_primitives::{
    Action as EosAction,
    BlockHeader as EosBlockHeader,
    ActionReceipt as EosActionReceipt,
};

use crate::btc_on_eos::eos::eos_crypto::eos_signature::EosSignature;

pub type EosAmount = String;
pub type EosAddress = String;
pub type MerkleProof = Vec<String>;
pub type EosAddresses = Vec<String>;
pub type EosAmounts = Vec<EosAmount>;
pub type EosSignatures = Vec<EosSignature>;
pub type Sha256HashedMessage = secp256k1::Message;
pub type EosSignedTransactions= Vec<EosSignedTransaction>;

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
    pub action_proofs: Vec<String>, // TODO (when we know the format!)
    pub block_header: EosBlockHeader,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterialJson {
    pub block_header: EosBlockHeaderJson,
    pub action_proofs: Vec<String>, // TODO (when we know the format!)
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
/*
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosBlockAndActionJson {
    pub block_header: Vec<String>, // TODO (when we know the format!)
    pub action_proofs: Vec<String>, // TODO (when we know the format!)
    //pub action: EosActionJson,
    //pub block_proof: Vec<String>, // TODO (when we know the format!)
    //pub receipt: EosActionReceiptJson,
    //pub raw_data: EosActionRawDataJson,
}
*/

/*
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosActionRawDataJson {
    pub nonce: usize,
    pub sender: String,
    pub receiver: String,
    pub quantity: String,
    pub ethereum_sender_str: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosAuthorizationJson {
    pub actor: String,
    pub permission: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosRawDataJson {
    pub nonce: String,
    pub sender: String,
    pub receiver: String,
    pub quantity: String,
    pub ethereum_sender_str: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosAuthSequenceJson {
    pub name: String,
    pub num: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosActionReceiptJson {
    pub receiver: String,
    pub act_digest: String,
    pub global_sequence: u64,
    pub recv_sequence: u64,
    pub auth_sequence: Vec<EosAuthSequenceJson>,
    pub code_sequence: usize,
    pub abi_sequence: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosActionJson {
    pub data: String,
    pub name: String,
    pub account: String,
    //pub raw_data: EosRawDataJson, // TODO Include this!
    pub authorization: Vec<EosAuthorizationJson>,
}

*/
