use std::str::FromStr;

use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::H256 as EthHash;

use crate::{
    rpc_server::{type_aliases::RpcParams, RpcCalls, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_challenge(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;
        let hash = EthHash::from_str(&checked_params[0])?;
        call_core(
            STRONGBOX_TIMEOUT,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetChallenge(hash),
        )
        .await
    }
}
