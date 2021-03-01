use derive_more::{Constructor, Deref, DerefMut};
use serde_json::{json, Value as JsonValue};

use crate::{
    chains::eos::{
        eos_constants::PROCESSED_TX_IDS_KEY,
        eos_database_utils::{get_processed_global_sequences_from_db, put_processed_tx_ids_in_db},
        eos_state::EosState,
    },
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};

pub type GlobalSequence = u64;

#[derive(Clone, Debug, PartialEq, Eq, Constructor, Deref, DerefMut)]
pub struct GlobalSequences(Vec<GlobalSequence>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Deref, DerefMut, Constructor)]
pub struct ProcessedGlobalSequences(pub Vec<GlobalSequence>);

impl ProcessedGlobalSequences {
    fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(self)?)
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn add(mut self, global_sequence: GlobalSequence) -> Self {
        self.push(global_sequence);
        self
    }

    pub fn add_multi(mut self, global_sequences: &mut GlobalSequences) -> Result<Self> {
        self.append(global_sequences);
        Ok(self)
    }

    pub fn to_json(&self) -> JsonValue {
        json!({"processed_global_sequences":self.0})
    }

    pub fn get_from_db<D: DatabaseInterface>(db: &D) -> Result<Self> {
        info!("✔ Getting EOS processed actions from db...");
        db.get(PROCESSED_TX_IDS_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|ref bytes| Self::from_bytes(bytes))
    }

    pub fn put_in_db<D: DatabaseInterface>(&self, db: &D) -> Result<()> {
        info!("✔ Putting EOS processed tx IDs in db...");
        db.put(
            PROCESSED_TX_IDS_KEY.to_vec(),
            self.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn remove(mut self, global_sequence: &GlobalSequence) -> Self {
        self.retain(|item| item != global_sequence);
        self
    }

    fn remove_global_sequence_from_list_in_db<D: DatabaseInterface>(
        db: &D,
        global_sequence: &GlobalSequence,
    ) -> Result<()> {
        info!(
            "✔ Removing global sequence: '{}' from `ProcessedGlobalSequences` in db...",
            global_sequence
        );
        Self::get_from_db(db)
            .map(|list| list.remove(global_sequence))
            .and_then(|updated_list| updated_list.put_in_db(db))
    }

    fn add_global_sequence_to_list_in_db<D: DatabaseInterface>(db: &D, global_sequence: GlobalSequence) -> Result<()> {
        info!(
            "✔ Adding global sequence: '{}' from `ProcessedGlobalSequences` in db...",
            global_sequence
        );
        Self::get_from_db(db)
            .map(|list| list.add(global_sequence))
            .and_then(|updated_list| updated_list.put_in_db(db))
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

#[cfg(test)]
mod teets {
    use super::*;
    use crate::test_utils::get_test_database;

    fn get_sample_processed_global_sequence_list() -> ProcessedGlobalSequences {
        ProcessedGlobalSequences::new(vec![])
            .add_multi(&mut GlobalSequences::new(vec![1u64, 2u64, 3u64]))
            .unwrap()
    }

    #[test]
    fn should_remove_extant_glob_sequence() {
        let list = get_sample_processed_global_sequence_list();
        let global_sequence = 2u64;
        let result = list.remove(&global_sequence);
        assert!(!result.contains(&global_sequence));
    }

    #[test]
    fn should_not_remove_non_extant_glob_sequence() {
        let list = get_sample_processed_global_sequence_list();
        let global_sequence = 5u64;
        assert!(!list.contains(&global_sequence));
        let result = list.remove(&global_sequence);
        assert_eq!(result, get_sample_processed_global_sequence_list());
    }

    #[test]
    fn should_make_to_and_from_bytes_roundtrip() {
        let list = get_sample_processed_global_sequence_list();
        let bytes = list.to_bytes().unwrap();
        let result = ProcessedGlobalSequences::from_bytes(&bytes).unwrap();
        assert_eq!(result, list);
    }

    #[test]
    fn should_put_and_get_processed_list_to_and_from_db() {
        let db = get_test_database();
        let list = get_sample_processed_global_sequence_list();
        list.put_in_db(&db).unwrap();
        let result = ProcessedGlobalSequences::get_from_db(&db).unwrap();
        assert_eq!(result, list);
    }

    #[test]
    fn should_remove_global_sequence_from_list_in_db() {
        let db = get_test_database();
        let list = get_sample_processed_global_sequence_list();
        let global_sequence = 2u64;
        list.put_in_db(&db).unwrap();
        ProcessedGlobalSequences::remove_global_sequence_from_list_in_db(&db, &global_sequence).unwrap();
        let result = ProcessedGlobalSequences::get_from_db(&db).unwrap();
        assert!(!result.contains(&global_sequence));
    }

    #[test]
    fn should_add_single_glob_sequence_to_list() {
        let list = get_sample_processed_global_sequence_list();
        let global_sequence = 1337u64;
        let result = list.add(global_sequence);
        assert!(result.contains(&global_sequence));
    }

    #[test]
    fn should_add_multi_glob_sequences_to_list() {
        let list = get_sample_processed_global_sequence_list();
        let global_sequence_1 = 1337u64;
        let global_sequence_2 = 1338u64;
        let mut global_sequences = GlobalSequences::new(vec![global_sequence_1, global_sequence_2]);
        let result = list.add_multi(&mut global_sequences).unwrap();
        assert!(result.contains(&global_sequence_1));
        assert!(result.contains(&global_sequence_2));
    }

    #[test]
    fn should_add_global_sequence_to_list_in_db() {
        let db = get_test_database();
        let list = get_sample_processed_global_sequence_list();
        let global_sequence = 1337u64;
        assert!(!list.contains(&global_sequence));
        list.put_in_db(&db).unwrap();
        ProcessedGlobalSequences::add_global_sequence_to_list_in_db(&db, global_sequence).unwrap();
        let result = ProcessedGlobalSequences::get_from_db(&db).unwrap();
        assert!(result.contains(&global_sequence));
    }
}
