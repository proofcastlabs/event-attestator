use function_name::named;

use crate::{
    chains::eos::{
        core_initialization::check_eos_core_is_initialized::check_eos_core_is_initialized,
        disable_protocol_feature::disable_feature_and_return_state,
        eos_database_transactions::end_eos_db_transaction_and_return_state,
        eos_database_utils::EosDbUtils,
        eos_state::EosState,
        get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
    },
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    traits::DatabaseInterface,
    types::Result,
};

/// Debug Disable Eos Protocol Feature
///
/// Remove an EOS protocol feature from the enable feature set.
#[named]
pub fn debug_disable_eos_protocol_feature<D: DatabaseInterface>(
    db: &D,
    feature_hash: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Maybe disabling EOS protocol feature w/ hash: {}", feature_hash);
    let hash = hex::decode(feature_hash)?;
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), &feature_hash, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| check_eos_core_is_initialized(&EosDbUtils::new(db)))
        .and_then(|_| get_enabled_protocol_features_and_add_to_state(EosState::init(db)))
        .and_then(|state| disable_feature_and_return_state(state, &hash))
        .and_then(end_eos_db_transaction_and_return_state)
        .map(|_| "{feature_disabled_success:true}".to_string())
}
