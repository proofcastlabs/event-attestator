use common::BridgeSide;
use common_sentinel::{Challenge, ConfigT, EthRpcMessages, SentinelConfig, SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::{
    rpc_server::{RpcCall, RpcParams},
    type_aliases::{EthRpcTx, WebSocketTx},
};

impl RpcCall {
    pub(crate) async fn handle_get_challenge_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling get challenge state...");
        let checked_params = Self::check_params(params, 1)?;

        // NOTE: Core cxn checked for us in list handler
        let challenge = Challenge::try_from(Self::handle_get_challenge(websocket_tx, checked_params, core_cxn).await?)?;

        let mcid = challenge.mcid();

        // NOTE We're still stuck with host and native for now, so we need to figure out which of
        // those this challenge originated.
        let side = if config.native().mcid() == *mcid {
            warn!("using bridge side NATIVE");
            BridgeSide::Native
        } else {
            warn!("using bridge side HOST");
            BridgeSide::Host
        };

        let (msg, rx) = EthRpcMessages::get_challenge_state_msg(side, challenge, config.pnetwork_hub(&side));

        if side.is_native() {
            native_eth_rpc_tx.send(msg).await?;
        } else {
            host_eth_rpc_tx.send(msg).await?;
        };

        let state = rx.await??;

        Ok(WebSocketMessagesEncodable::Success(json!(state)))
    }
}
