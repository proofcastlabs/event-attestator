use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chains::eos::parse_submission_material::parse_eos_submission_material_string_to_struct
};

pub fn parse_submission_material_and_add_to_state<D>(
    submission_material: &str,
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    parse_eos_submission_material_string_to_struct(submission_material)
        .and_then(|material| state.add_submission_material(material))
}
