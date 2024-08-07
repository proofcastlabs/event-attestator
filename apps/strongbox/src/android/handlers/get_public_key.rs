use common_eth::ChainDbUtils;
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_public_key(state: State) -> Result<State, SentinelError> {
    let j = json!({"publicKey": ChainDbUtils::new(state.db()).get_public_key()?.public_key.to_string()});

    let r = WebSocketMessagesEncodable::Success(j);
    Ok(state.add_response(r))
}
