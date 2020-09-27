pub use eos_primitives::Checksum256;
use eos_primitives::AccountName as EosAccountName;
use crate::{
    types::Result,
    chains::eos::{
        eos_types::{
            GlobalSequence,
            GlobalSequences,
        },
        eos_action_proofs::EosActionProof,
        parse_redeem_infos::{
            get_eos_amount_from_action_data,
            get_redeem_address_from_action_data,
            get_redeem_action_sender_from_action_data,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcOnEosRedeemInfo {
    pub amount: u64,
    pub recipient: String,
    pub from: EosAccountName,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
}

impl BtcOnEosRedeemInfo {
    pub fn from_action_proof(action_proof: &EosActionProof) -> Result<Self> {
        Ok(
            BtcOnEosRedeemInfo {
                originating_tx_id: action_proof.tx_id,
                global_sequence: action_proof.action_receipt.global_sequence,
                amount: get_eos_amount_from_action_data(&action_proof.action.data)?,
                from: get_redeem_action_sender_from_action_data(&action_proof.action.data)?,
                recipient: get_redeem_address_from_action_data(&action_proof.action.data)?,
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcOnEosRedeemInfos(pub Vec<BtcOnEosRedeemInfo>);

impl BtcOnEosRedeemInfos {
    pub fn new(redeem_infos: &[BtcOnEosRedeemInfo]) -> Self {
        Self(redeem_infos.to_vec())
    }

    pub fn sum(&self) -> u64 {
        self.0.iter().fold(0, |acc, infos| acc + infos.amount)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_global_sequences(&self) -> GlobalSequences {
        self.0.iter().map(|infos| infos.global_sequence).collect()
    }
}

