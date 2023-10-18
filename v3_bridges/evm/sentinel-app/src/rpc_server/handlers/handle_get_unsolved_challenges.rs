use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCall, STRONGBOX_TIMEOUT_MS},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_get_unsolved_challenges(
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        call_core(
            STRONGBOX_TIMEOUT_MS,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetUnsolvedChallenges,
        )
        .await
    }
}
