use crate::{
    traits::DatabaseInterface,
    types::{Byte, Result},
    chains::eos::{
        enable_protocol_feature::enable_protocol_feature,
        eos_database_utils::{
            end_eos_db_transaction,
            start_eos_db_transaction,
        },
    },
    btc_on_eos::{
        check_core_is_initialized::check_core_is_initialized_and_return_eos_state,
        eos::{
            eos_state::EosState,
            get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
        },
    },
};

fn enable_feature_and_return_state<D>(
    state: EosState<D>,
    hash: &[Byte],
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    enable_protocol_feature(&state.db, hash, &state.enabled_protocol_features).and(Ok(state))
}

pub fn enable_eos_protocol_feature<D: DatabaseInterface>(db: D, feature_hash: &str) -> Result<String> {
    info!("✔ Maybe enabling EOS protocol feature w/ hash: {}", feature_hash);
    let hash = hex::decode(feature_hash)?;
    check_core_is_initialized_and_return_eos_state(EosState::init(db))
        .and_then(start_eos_db_transaction)
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(|state| enable_feature_and_return_state(state, &hash))
        .and_then(end_eos_db_transaction)
        .map(|_| "{feature_enabled_success:true}".to_string())
}
