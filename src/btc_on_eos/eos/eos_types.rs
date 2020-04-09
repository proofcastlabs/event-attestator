use std::fmt;
use eos_primitives::{
    Checksum256,
    Action as EosAction,
    AccountName as EosAccountName,
    BlockHeader as EosBlockHeader,
    ProducerKey as EosProducerKey,
    ActionReceipt as EosActionReceipt,
};
use crate::btc_on_eos::{
    utils::convert_hex_to_checksum256,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_utils::get_eos_schedule_db_key,
        eos_crypto::eos_signature::EosSignature,
        parse_eos_actions::parse_eos_action_json,
        parse_eos_action_receipts::parse_eos_action_receipt_json,
    },
};

pub type EosAmount = String;
pub type EosAddress = String;
pub type MerklePath = Vec<Bytes>;
pub type MerkleProof = Vec<String>;
pub type EosAddresses = Vec<String>;
pub type EosAmounts = Vec<EosAmount>;
pub type Checksum256s = Vec<Checksum256>;
pub type ActionProofs = Vec<ActionProof>;
pub type MerkleProofs = Vec<MerkleProof>;
pub type EosSignatures = Vec<EosSignature>;
pub type ProducerKeys = Vec<EosProducerKey>;
pub type ActionProofJsons = Vec<ActionProofJson>;
pub type Sha256HashedMessage = secp256k1::Message;
pub type AuthSequenceJsons = Vec<AuthSequenceJson>;
pub type AuthorizationJsons = Vec<AuthorizationJson>;
pub type EosSignedTransactions = Vec<EosSignedTransaction>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EosKnownSchedules(Vec<EosKnownSchedule>);

impl EosKnownSchedules {
    pub fn new(version: u32) -> Self {
        EosKnownSchedules(vec![EosKnownSchedule::new(version)])
    }

    pub fn add(mut self, version: u32) -> Self {
        self.0.push(EosKnownSchedule::new(version));
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EosKnownSchedule {
    pub schedule_db_key: Bytes,
    pub schedule_version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EosKnownSchedulesJsons(Vec<EosKnownScheduleJson>);

impl EosKnownSchedulesJsons {
    pub fn from_schedules(scheds: EosKnownSchedules) -> EosKnownSchedulesJsons {
        EosKnownSchedulesJsons(
            scheds
                .0
                .iter()
                .map(|sched| EosKnownScheduleJson::from_schedule(sched))
                .collect::<Vec<EosKnownScheduleJson>>()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EosKnownScheduleJson {
    pub schedule_db_key: String,
    pub schedule_version: u32,
}

impl EosKnownScheduleJson {
    pub fn from_schedule(sched: &EosKnownSchedule) -> Self {
        EosKnownScheduleJson {
            schedule_version: sched.schedule_version.clone(),
            schedule_db_key: hex::encode(sched.schedule_db_key.clone()),
        }
    }
}

impl EosKnownSchedule {
    pub fn new(version: u32) -> Self {
        EosKnownSchedule {
            schedule_version: version.clone(),
            schedule_db_key: get_eos_schedule_db_key(version),
        }
    }
}

impl fmt::Display for EosKnownSchedules {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EosKnownSchedule:")?;
        for v in &self.0 {
            write!(f, "{}", v)?;
        }
        Ok(())
    }
}

impl fmt::Display for EosKnownSchedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tschedule_version: {},\n\tdb_key: {}",
            self.schedule_version,
            hex::encode(&self.schedule_db_key)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedeemParams {
    pub amount: u64,
    pub recipient: String,
    pub from: EosAccountName,
    pub originating_tx_id: Checksum256,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum EosNetwork {
    Mainnet,
    Testnet,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct EosSignedTransaction {
    pub amount: String,
    pub recipient: String,
    pub signature: String,
    pub transaction: String,
}

impl EosSignedTransaction {
    pub fn new(
        signature: String,
        transaction: String,
        recipient: String,
        amount: String,
    ) -> EosSignedTransaction {
        EosSignedTransaction {
            signature,
            transaction,
            amount,
            recipient,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterial {
    pub producer_signature: String,
    pub action_proofs: ActionProofs,
    pub block_header: EosBlockHeader,
    pub blockroot_merkle: Checksum256s,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EosSubmissionMaterialJson {
    pub blockroot_merkle: Vec<String>,
    pub action_proofs: ActionProofJsons,
    pub block_header: EosBlockHeaderJson,
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
pub struct ProducerSchedule {
    pub version: u32,
    pub producers: ProducerKeys,
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
    pub action_proof: MerkleProof,
    pub action_receipt: EosActionReceipt,
}

impl ActionProof {
    pub fn from_json(json: &ActionProofJson) -> Result<Self> {
        Ok(
            ActionProof {
                action_proof:
                    json.action_proof.clone(),
                tx_id:
                    convert_hex_to_checksum256(&json.tx_id)?,
                action:
                    parse_eos_action_json(&json.action_json)?,
                action_receipt:
                    parse_eos_action_receipt_json(&json.action_receipt_json)?,
            }
        )
    }
}

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

    pub fn add_multi(mut self, tx_ids: &mut Vec<String>) -> Result<Self> {
        self.0.append(tx_ids);
        Ok(self)
    }

    pub fn contains(&self, tx_id: &String) -> bool {
        self.0.contains(tx_id)
    }
}
