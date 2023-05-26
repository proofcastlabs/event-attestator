use common::{constants::ZERO_CONFS_WARNING, traits::DatabaseInterface, types::Result};
use serde_json::json;

use crate::BtcState;

pub fn get_btc_init_output_json<D: DatabaseInterface>(state: BtcState<D>) -> Result<String> {
    let number_of_confirmations = state.btc_db_utils.get_btc_canon_to_tip_length_from_db()?;
    Ok(json!({
        "btc_address": state.btc_db_utils.get_btc_address_from_db()?,
        "btc_latest_block_num": state.btc_db_utils.get_latest_btc_block_number()?,
        "number_of_confirmations":
        if number_of_confirmations == 0 {
            ZERO_CONFS_WARNING.to_string()
        } else {
            number_of_confirmations.to_string()
        },
    })
    .to_string())
}
