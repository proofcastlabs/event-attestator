use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::btc::{
        btc_state::BtcState,
        btc_types::BtcSubmissionMaterial,
    },
};

pub fn parse_submission_material_and_put_in_state<D: DatabaseInterface>(
    json_str: &str,
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    info!("✔ Parsing BTC submisson material...");
    BtcSubmissionMaterial::from_str(&json_str).and_then(|result| state.add_btc_submission_material(result))
}
