use common::{traits::DatabaseInterface, types::Result};

use crate::eos_global_sequences::ProcessedGlobalSequences;

/// # Get Processed Actions List
///
/// This function returns the list of already-processed action global sequences in JSON format.
pub fn get_processed_actions_list<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting processed actions list...");
    db.start_transaction()
        .and_then(|_| ProcessedGlobalSequences::get_from_db(db))
        .and_then(|processed_global_sequences| {
            db.end_transaction()?;
            Ok(processed_global_sequences.to_json().to_string())
        })
}
