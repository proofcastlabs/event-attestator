use crate::{
    chains::eos::{
        eos_action_proofs::EosActionProof,
        eos_state::EosState,
        eos_types::{GlobalSequence, GlobalSequences, ProcessedTxIds},
    },
    traits::DatabaseInterface,
    types::Result,
};
use derive_more::{Constructor, Deref};
use eos_primitives::{AccountName as EosAccountName, Checksum256};
use ethereum_types::{Address as EthAddress, U256};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Constructor)]
pub struct EosOnEthEosTxInfo {
    pub amount: U256,
    pub from: EosAccountName,
    pub recipient: EthAddress,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Deref, Constructor)]
pub struct EosOnEthEosTxInfos(pub Vec<EosOnEthEosTxInfo>);

impl EosOnEthEosTxInfos {
    pub fn get_global_sequences(&self) -> GlobalSequences {
        self.iter().map(|infos| infos.global_sequence).collect()
    }

    pub fn from_action_proofs(action_proofs: &[EosActionProof]) -> Result<EosOnEthEosTxInfos> {
        Ok(EosOnEthEosTxInfos::new(
            action_proofs
                .iter()
                .map(|action_proof| action_proof.to_eos_on_eth_eos_tx_info())
                .collect::<Result<Vec<EosOnEthEosTxInfo>>>()?,
        ))
    }

    pub fn filter_out_already_processed_txs(&self, processed_tx_ids: &ProcessedTxIds) -> Result<Self> {
        Ok(EosOnEthEosTxInfos::new(
            self.iter()
                .filter(|info| !processed_tx_ids.contains(&info.global_sequence))
                .cloned()
                .collect::<Vec<EosOnEthEosTxInfo>>(),
        ))
    }
}

pub fn maybe_parse_eos_on_eth_eos_tx_infos_and_put_in_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Parsing redeem params from actions data...");
    EosOnEthEosTxInfos::from_action_proofs(&state.action_proofs).and_then(|tx_infos| {
        info!("✔ Parsed {} sets of redeem info!", tx_infos.len());
        state.add_eos_on_eth_eos_tx_info(tx_infos)
    })
}

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Filtering out already processed tx IDs...");
    state
        .eos_on_eth_eos_tx_infos
        .filter_out_already_processed_txs(&state.processed_tx_ids)
        .and_then(|filtered| state.add_eos_on_eth_eos_tx_info(filtered))
}
