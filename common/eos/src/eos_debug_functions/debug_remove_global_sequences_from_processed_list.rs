use std::str::FromStr;

use common::{
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use serde_json::json;

use crate::eos_global_sequences::{GlobalSequences, ProcessedGlobalSequences};

/// Debug Remove Global Sequence From Processed List
///
/// This function will remove a global sequence from the list of processed ones stored in the
/// encrypted database. This allows a debug user to override the replay protection this list
/// provides. Use with caution!
#[named]
pub fn debug_remove_global_sequences_from_processed_list<D: DatabaseInterface>(
    db: &D,
    global_sequences_json: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug adding global sequences to processed list...");
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), global_sequences_json, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
        .and_then(|_| {
            ProcessedGlobalSequences::remove_global_sequences_from_list_in_db(
                db,
                &GlobalSequences::from_str(global_sequences_json)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(
            json!({"removed_global_sequences_to_processed_list":true}).to_string()
        ))
        .map(prepend_debug_output_marker_to_string)
}
