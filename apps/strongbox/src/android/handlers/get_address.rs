use common_eth::ChainDbUtils;
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_address(state: State) -> Result<State, SentinelError> {
    let j = json!({"address": ChainDbUtils::new(state.db()).get_signing_address()?});

    let r = WebSocketMessagesEncodable::Success(j);
    Ok(state.add_response(r))
}
