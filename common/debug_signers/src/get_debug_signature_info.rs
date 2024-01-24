use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::convert_hex_to_h256;
use serde_json::Value as JsonValue;

use crate::{DebugSignatories, SAFE_DEBUG_SIGNATORIES};

/// Get Debug Signature Info
///
/// Gets the information required to sign a valid debug function signature, required in order to
/// run a debug function. The `debug_command_hash_str` is a hash of the `CLI_ARGS` struct
/// populated with the arguments required to run the desired debug function.
pub fn get_debug_signature_info<D: DatabaseInterface>(
    db: &D,
    core_type: &CoreType,
    debug_command_hash_str: &str,
    use_safe_debug_signers: bool,
) -> Result<JsonValue> {
    if !cfg!(feature = "skip-db-transactions") {
        db.start_transaction()?
    };

    DebugSignatories::get_from_db(db).and_then(|debug_signatories| {
        if !cfg!(feature = "skip-db-transactions") {
            db.end_transaction()?
        };
        let debug_command_hash = convert_hex_to_h256(debug_command_hash_str)?;
        if debug_signatories.is_empty() {
            let msg = "using safe debug signers to validate signature info";
            if use_safe_debug_signers {
                debug!("{msg}");
                SAFE_DEBUG_SIGNATORIES.to_signature_info_json(core_type, &debug_command_hash, None)
            } else {
                debug!("not {msg}");
                DebugSignatories::default().to_signature_info_json(core_type, &debug_command_hash, None)
            }
        } else {
            debug_signatories.to_signature_info_json(core_type, &debug_command_hash, None)
        }
    })
}
