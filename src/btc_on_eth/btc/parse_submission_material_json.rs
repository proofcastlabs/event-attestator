use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::btc::{
        btc_state::BtcState,
        btc_types::BtcSubmissionMaterialJson
    },
};

pub fn parse_btc_block_string_to_json(btc_block_json_string: &str) -> Result<BtcSubmissionMaterialJson> {
    trace!("✔ Parsing JSON string to `BtcSubmissionMaterialJson`...");
    match serde_json::from_str(btc_block_json_string) {
        Ok(json) => Ok(json),
        Err(err) => Err(err.into())
    }
}

pub fn parse_btc_submission_json_and_put_in_state<D>(
    block_json: &str,
    state: BtcState<D>,
) -> Result<BtcState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing BTC submission json...");
    parse_btc_block_string_to_json(&block_json).and_then(|result| state.add_btc_submission_json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::btc::btc_test_utils::get_sample_btc_submission_material_json_string;

    #[test]
    fn should_parse_btc_block_json() {
        let string = get_sample_btc_submission_material_json_string();
        parse_btc_block_string_to_json(&string).unwrap();
    }
}
