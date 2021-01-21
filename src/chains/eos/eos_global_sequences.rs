use crate::{
    chains::eos::{
        eos_database_utils::{get_processed_global_sequences_from_db, put_processed_tx_ids_in_db},
        eos_state::EosState,
    },
    traits::DatabaseInterface,
    types::Result,
};
use derive_more::{Constructor, Deref, DerefMut};
use serde_json::{json, Value as JsonValue};

pub type GlobalSequence = u64;

#[derive(Clone, Debug, PartialEq, Eq, Constructor, Deref, DerefMut)]
pub struct GlobalSequences(Vec<GlobalSequence>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessedGlobalSequences(pub Vec<GlobalSequence>);

impl ProcessedGlobalSequences {
    pub fn init() -> Self {
        ProcessedGlobalSequences(vec![])
    }

    pub fn add_multi(mut self, global_sequences: &mut GlobalSequences) -> Result<Self> {
        self.0.append(global_sequences);
        Ok(self)
    }

    pub fn contains(&self, global_sequence: &GlobalSequence) -> bool {
        self.0.contains(global_sequence)
    }

    pub fn to_json(&self) -> JsonValue {
        json!({"processed_global_sequences":self.0})
    }

    pub fn get_from_db<D: DatabaseInterface>(db: &D) -> Result<Self> {
        get_processed_global_sequences_from_db(db)
    }
}

pub fn maybe_add_global_sequences_to_processed_list_and_return_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    let mut global_sequences = state.get_global_sequences();
    match global_sequences.len() {
        0 => {
            info!("✔ No `global_sequences` to add to processed tx list!");
            Ok(state)
        },
        _ => {
            info!("✔ Adding `global_sequences` to processed tx list...");
            put_processed_tx_ids_in_db(
                &state.db,
                &state.processed_tx_ids.clone().add_multi(&mut global_sequences)?,
            )
            .and(Ok(state))
        },
    }
}

pub fn get_processed_global_sequences_and_add_to_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    get_processed_global_sequences_from_db(&state.db).and_then(|tx_ids| state.add_processed_tx_ids(tx_ids))
}
