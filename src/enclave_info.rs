use serde::{Deserialize, Serialize};

use crate::{
    constants::{CORE_IS_VALIDATING, DB_KEY_PREFIX, DEBUG_MODE},
    fees::fee_constants::DISABLE_FEES,
    utils::get_core_version,
};

#[derive(Serialize, Deserialize)]
pub struct EnclaveInfo {
    debug_mode: bool,
    db_key_prefix: String,
    core_is_validating: bool,
    core_version: String,
    fees_enabled: bool,
}

impl EnclaveInfo {
    pub fn new() -> Self {
        Self {
            debug_mode: DEBUG_MODE,
            core_is_validating: CORE_IS_VALIDATING,
            db_key_prefix: DB_KEY_PREFIX.to_string(),
            core_version: get_core_version(),
            fees_enabled: !DISABLE_FEES,
        }
    }
}
