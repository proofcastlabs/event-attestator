use derive_more::{
    Deref,
    Constructor,
};
use eos_primitives::{
    Checksum256,
    AccountName as EosAccountName,
};
use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        eos_action_proofs::EosActionProof,
        eos_types::{
            GlobalSequence,
            GlobalSequences,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct BtcOnEosRedeemInfos(pub Vec<BtcOnEosRedeemInfo>);

impl BtcOnEosRedeemInfos {
    pub fn sum(&self) -> u64 {
        self.0.iter().fold(0, |acc, infos| acc + infos.amount)
    }

    pub fn get_global_sequences(&self) -> GlobalSequences {
        self.0.iter().map(|infos| infos.global_sequence).collect()
    }

    pub fn from_action_proofs(action_proofs: &[EosActionProof]) -> Result<BtcOnEosRedeemInfos> {
        Ok(BtcOnEosRedeemInfos::new(
            action_proofs
                .iter()
                .map(|action_proof| action_proof.to_btc_on_eos_redeem_info())
                .collect::<Result<Vec<BtcOnEosRedeemInfo>>>()?
        ))
    }
}

pub fn maybe_parse_redeem_infos_and_put_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing redeem params from actions data...");
    BtcOnEosRedeemInfos::from_action_proofs(&state.action_proofs)
        .and_then(|redeem_infos| {
            info!("✔ Parsed {} sets of redeem info!", redeem_infos.len());
            state.add_btc_on_eos_redeem_infos(redeem_infos)
        })
}
