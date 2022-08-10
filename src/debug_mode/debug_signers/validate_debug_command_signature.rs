use std::str::FromStr;

use crate::{
    chains::eth::{eth_crypto::eth_signature::EthSignature, eth_utils::convert_hex_to_h256},
    core_type::CoreType,
    debug_mode::debug_signers::debug_signatories::DebugSignatories,
    traits::DatabaseInterface,
    types::Result,
};

/// Validate Debug Command Signature
///
/// This function will take in the passed debug command hash, signature and database and check that
/// the signature is valid for one of the debug signatory's over that command hash.
pub fn validate_debug_command_signature<D: DatabaseInterface>(
    db: &D,
    core_type: &CoreType,
    signature: &str,
    debug_command_hash: &str,
) -> Result<()> {
    if cfg!(test) {
        warn!("✘ Skipping debug command validation!");
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
