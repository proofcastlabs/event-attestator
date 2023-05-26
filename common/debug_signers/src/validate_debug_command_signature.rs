use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::{convert_hex_to_h256, EthSignature};

use crate::DebugSignatories;

/// Validate Debug Command Signature
///
/// This function will take in the passed debug command hash, signature and database and check that
/// the signature is valid for one of the debug signatory's over that command hash.
pub fn validate_debug_command_signature<D: DatabaseInterface>(
    db: &D,
    core_type: &CoreType,
    signature: &str,
    debug_command_hash: &str,
    is_test: bool,
) -> Result<()> {
    if is_test {
        warn!("✘ Skipping debug signature check!");
        Ok(())
    } else {
        DebugSignatories::get_from_db(db).and_then(|debug_signatories| {
            debug_signatories.maybe_validate_signature_and_increment_nonce_in_db(
                db,
                core_type,
                &convert_hex_to_h256(debug_command_hash)?,
                &EthSignature::from_str(signature)?,
            )
        })
    }
}
