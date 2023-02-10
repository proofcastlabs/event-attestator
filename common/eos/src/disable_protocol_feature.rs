use common::{
    traits::DatabaseInterface,
    types::{Byte, Result},
};

use crate::{
    eos_database_utils::EosDbUtils,
    protocol_features::{EnabledFeatures, AVAILABLE_FEATURES},
    EosState,
};

fn disable_protocol_feature<D: DatabaseInterface>(
    db_utils: &EosDbUtils<D>,
    feature_hash: &[Byte],
    enabled_features: &EnabledFeatures,
) -> Result<()> {
    AVAILABLE_FEATURES.check_contains(feature_hash).and_then(|_| {
        if enabled_features.is_not_enabled(feature_hash) {
            return Err("✘ Feature not enabled, doing nothing!".into());
        }
        info!("✔ Disabling feature: {}", hex::encode(feature_hash));
        enabled_features
            .clone()
            .remove(feature_hash)
            .and_then(|new_features| db_utils.put_eos_enabled_protocol_features_in_db(&new_features))
    })
}

pub fn disable_feature_and_return_state<'a, D: DatabaseInterface>(
    state: EosState<'a, D>,
    hash: &[Byte],
) -> Result<EosState<'a, D>> {
    disable_protocol_feature(&state.eos_db_utils, hash, &state.enabled_protocol_features).and(Ok(state))
}
