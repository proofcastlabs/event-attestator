use crate::{state::EosState, traits::DatabaseInterface, types::Result};

pub fn get_enabled_protocol_features_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Getting enabled EOS protocol features and adding to state...");
    state
        .eos_db_utils
        .get_eos_enabled_protocol_features_from_db()
        .and_then(|schedule| state.add_enabled_protocol_features(schedule))
}
