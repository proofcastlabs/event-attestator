use common::{
    chains::eos::{
        enable_protocol_feature::enable_feature_and_return_state,
        eos_database_transactions::end_eos_db_transaction_and_return_state,
        get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
    },
    core_type::CoreType,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};
use debug_signers::validate_debug_command_signature;
use function_name::named;

/// Debug Enable Eos Protocol Feature
///
/// Enables the supplied protocol feature.
#[named]
pub fn debug_enable_eos_protocol_feature<D: DatabaseInterface>(
    db: &D,
    feature_hash: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Maybe enabling EOS protocol feature w/ hash: {}", feature_hash);
    let hash = hex::decode(feature_hash)?;
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &feature_hash, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_enabled_protocol_features_and_add_to_state(EosState::init(db)))
        .and_then(|state| enable_feature_and_return_state(state, &hash))
        .and_then(end_eos_db_transaction_and_return_state)
        .map(|_| "{feature_enabled_success:true}".to_string())
}
