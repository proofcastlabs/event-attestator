use crate::types::Result;
use derive_more::{Constructor, Deref, DerefMut};
use serde_json::{json, Value as JsonValue};

pub type GlobalSequence = u64;

#[derive(Clone, Debug, PartialEq, Eq, Constructor, Deref, DerefMut)]
pub struct GlobalSequences(Vec<GlobalSequence>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessedTxIds(pub Vec<GlobalSequence>);

impl ProcessedTxIds {
    pub fn init() -> Self {
        ProcessedTxIds(vec![])
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
}
