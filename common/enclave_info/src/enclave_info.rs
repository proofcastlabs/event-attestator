use common::{
    constants::{CORE_IS_VALIDATING, DB_KEY_PREFIX, MAX_FEE_BASIS_POINTS},
    traits::DatabaseInterface,
    types::Result,
    utils::get_core_version,
};
use common_debug_signers::DebugSignatories;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Serialize, Deserialize)]
pub struct EnclaveInfo {
    db_key_prefix: String,
    core_is_validating: bool,
    core_version: String,
    max_fee_basis_points: u64,
    debug_signatories: JsonValue,
}

impl EnclaveInfo {
    pub fn new<D: DatabaseInterface>(db: &D) -> Result<Self> {
        Ok(Self {
            core_version: get_core_version(),
            core_is_validating: CORE_IS_VALIDATING,
            db_key_prefix: DB_KEY_PREFIX.to_string(),
            max_fee_basis_points: MAX_FEE_BASIS_POINTS,
            debug_signatories: DebugSignatories::get_from_db(db)?.to_enclave_state_json(),
        })
    }
}
