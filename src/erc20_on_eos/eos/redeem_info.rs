use ethereum_types::{
    U256,
    Address as EthAddress,
};
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
pub struct Erc20OnEosRedeemInfo {
    pub amount: U256,
    pub from: EosAccountName,
    pub recipient: EthAddress,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct Erc20OnEosRedeemInfos(pub Vec<Erc20OnEosRedeemInfo>);

impl Erc20OnEosRedeemInfos {
    pub fn sum(&self) -> U256 {
        self.0.iter().fold(U256::zero(), |acc, infos| acc + infos.amount)
    }

    pub fn get_global_sequences(&self) -> GlobalSequences {
        self.0.iter().map(|infos| infos.global_sequence).collect()
    }

    pub fn from_action_proofs(action_proofs: &[EosActionProof]) -> Result<Erc20OnEosRedeemInfos> {
        Ok(Erc20OnEosRedeemInfos::new(
            action_proofs
                .iter()
                .map(|action_proof| action_proof.to_erc20_on_eos_redeem_info())
                .collect::<Result<Vec<Erc20OnEosRedeemInfo>>>()?
        ))
    }
}

pub fn maybe_parse_redeem_infos_and_put_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing redeem params from actions data...");
    Erc20OnEosRedeemInfos::from_action_proofs(&state.action_proofs)
        .and_then(|redeem_infos| {
            info!("✔ Parsed {} sets of redeem info!", redeem_infos.len());
            state.add_erc20_on_eos_redeem_infos(redeem_infos)
        })
}
