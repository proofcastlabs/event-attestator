use crate::{
    chains::eos::{
        core_initialization::check_eos_core_is_initialized::check_eos_core_is_initialized,
        eos_database_transactions::end_eos_db_transaction_and_return_state,
        eos_database_utils::EosDbUtils,
        eos_state::EosState,
        get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
        protocol_features::{EnabledFeatures, AVAILABLE_FEATURES},
    },
    core_type::CoreType,
    debug_mode::validate_debug_command_signature,
    traits::DatabaseInterface,
    types::{Byte, Result},
};

fn enable_protocol_feature<D: DatabaseInterface>(
    db_utils: &EosDbUtils<D>,
    feature_hash: &[Byte],
    enabled_features: &EnabledFeatures,
) -> Result<()> {
    AVAILABLE_FEATURES.check_contains(feature_hash).and_then(|_| {
        if enabled_features.is_enabled(feature_hash) {
            return Err("✘ Feature already enabled, doing nothing!".into());
        }
        info!("✔ Enabling new feature: {}", hex::encode(feature_hash));
        enabled_features
            .clone()
            .add(feature_hash)
            .and_then(|new_features| db_utils.put_eos_enabled_protocol_features_in_db(&new_features))
    })
}

fn enable_feature_and_return_state<'a, D: DatabaseInterface>(
    state: EosState<'a, D>,
    hash: &[Byte],
) -> Result<EosState<'a, D>> {
    enable_protocol_feature(&state.eos_db_utils, hash, &state.enabled_protocol_features).and(Ok(state))
}

pub fn debug_enable_eos_protocol_feature<D: DatabaseInterface>(
    db: &D,
    feature_hash: &str,
    core_type: &CoreType,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Maybe enabling EOS protocol feature w/ hash: {}", feature_hash);
    let hash = hex::decode(feature_hash)?;
    check_eos_core_is_initialized(&EosDbUtils::new(db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, core_type, signature, debug_command_hash))
        .and_then(|_| get_enabled_protocol_features_and_add_to_state(EosState::init(db)))
        .and_then(|state| enable_feature_and_return_state(state, &hash))
        .and_then(end_eos_db_transaction_and_return_state)
        .map(|_| "{feature_enabled_success:true}".to_string())
}
