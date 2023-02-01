use serde_json::Value as JsonValue;

use crate::{
    chains::eth::eth_utils::convert_hex_to_h256,
    core_type::CoreType,
    debug_functions::debug_signers::debug_signatories::{DebugSignatories, SAFE_DEBUG_SIGNATORIES},
    traits::DatabaseInterface,
    types::Result,
};

/// Get Debug Signature Info
///
/// Gets the information required to sign a valid debug function signaure, require in order to run
/// a debug function. The `debug_command_hash_str` is a hash of the `CLI_ARGS` struct populated
/// with the arguments required to run the desired debug function.
pub fn get_debug_signature_info<D: DatabaseInterface>(
    db: &D,
    core_type: &CoreType,
    debug_command_hash_str: &str,
) -> Result<JsonValue> {
    db.start_transaction()
        .and_then(|_| DebugSignatories::get_from_db(db))
        .and_then(|debug_signatories| {
            db.end_transaction()?;
            let debug_command_hash = convert_hex_to_h256(debug_command_hash_str)?;
            if debug_signatories.is_empty() {
                // NOTE: If there are no signers yet, we show the safe address signing info, since
                // with that, new debug signers can be added.
                SAFE_DEBUG_SIGNATORIES.to_signature_info_json(core_type, &debug_command_hash, None)
            } else {
                debug_signatories.to_signature_info_json(core_type, &debug_command_hash, None)
            }
        })
}
