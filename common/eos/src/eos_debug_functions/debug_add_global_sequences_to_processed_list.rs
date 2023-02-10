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

/// Debug Add Global Sequence to Processed List
///
/// This function will add a global sequence to the list of processed ones stored in the encrypted
/// database. This will mean that the EOS action with that global sequence cannot be processed.
#[named]
pub fn debug_add_global_sequences_to_processed_list<D: DatabaseInterface>(
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
            ProcessedGlobalSequences::add_global_sequences_to_list_in_db(
                db,
                &mut GlobalSequences::from_str(global_sequences_json)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"added_global_sequences_to_processed_list":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}
