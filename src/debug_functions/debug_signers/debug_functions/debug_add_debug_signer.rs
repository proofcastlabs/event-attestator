use std::str::FromStr;

use function_name::named;
use serde_json::json;

use crate::{
    chains::eth::{
        eth_crypto::eth_signature::EthSignature,
        eth_utils::{convert_hex_to_eth_address, convert_hex_to_h256},
    },
    core_type::CoreType,
    debug_functions::debug_signers::{
        debug_signatories::{DebugSignatories, SAFE_DEBUG_SIGNATORIES},
        debug_signatory::DebugSignatory,
    },
    safe_addresses::SAFE_ETH_ADDRESS,
    traits::DatabaseInterface,
    types::Result,
};

/// Debug Add Debug Signer
///
/// Adds a debug signatory to the list. Since this is a debug function, it requires a valid
/// signature from an address in the list of debug signatories. But because this list begins life
/// empty, we have a chicken and egg scenario. And so to solve this, if the addition is the _first_
/// one, we instead require a signature from the `SAFE_ETH_ADDRESS` in order to validate the
/// command.
#[named]
pub fn debug_add_debug_signer<D: DatabaseInterface>(
    db: &D,
    signatory_name: &str,
    eth_address_str: &str,
    core_type: &CoreType,
    signature_str: &str,
) -> Result<String> {
    info!("✔ Adding debug signer to list...");

    let eth_address = convert_hex_to_eth_address(eth_address_str)?;
    if eth_address == *SAFE_ETH_ADDRESS {
        return Err(
            json!({"error": "Cannot add the ETH safe address as a debug signatory!"})
                .to_string()
                .into(),
        );
    };

    db.start_transaction()
        .and_then(|_| DebugSignatories::get_from_db(db))
        .and_then(|debug_signatories| {
            let debug_command_hash = convert_hex_to_h256(&get_debug_command_hash!(
                function_name!(),
                signatory_name,
                eth_address_str,
                core_type
            )()?)?;
            let signature = EthSignature::from_str(signature_str)?;
            let debug_signatory_to_add = DebugSignatory::new(signatory_name, &eth_address);

            if debug_signatories.is_empty() {
                info!("✔ Validating the debug signer addition using the safe address...");
                SAFE_DEBUG_SIGNATORIES
                    .maybe_validate_signature_and_increment_nonce_in_db(db, core_type, &debug_command_hash, &signature)
                    .and_then(|_| debug_signatories.add_and_update_in_db(db, &debug_signatory_to_add))
            } else {
                debug_signatories
                    .maybe_validate_signature_and_increment_nonce_in_db(db, core_type, &debug_command_hash, &signature)
                    .and_then(|_| DebugSignatories::get_from_db(db))
                    .and_then(|debug_signatories| debug_signatories.add_and_update_in_db(db, &debug_signatory_to_add))
            }
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"debug_add_signatory_success":true, "eth_address": eth_address_str}).to_string())
}
