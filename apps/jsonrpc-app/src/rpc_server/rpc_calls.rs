use common_sentinel::{
    Env,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use warp::{reject::Reject, Rejection};

use super::{
    type_aliases::{RpcId, RpcParams},
    JsonRpcRequest,
};
use crate::type_aliases::{BroadcastChannelTx, CoreCxnStatus, WebSocketTx};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error(String);

impl Reject for Error {}

fn create_json_rpc_response<T: Serialize>(id: RpcId, t: T) -> Json {
    json!({ "id": id, "result": t, "jsonrpc": "2.0" })
}

fn create_json_rpc_error(id: RpcId, code: u64, msg: &str) -> Json {
    json!({ "id": id, "error": { "code": code, "message": msg, }, "jsonrpc": "2.0" })
}

fn create_json_rpc_error_with_json<T: Serialize>(id: RpcId, code: u64, msg: &T) -> Json {
    json!({ "id": id, "error": { "code": code, "message": msg, }, "jsonrpc": "2.0" })
}

// FIXME make a type for error code
fn create_json_rpc_response_from_result<T: Serialize>(id: RpcId, r: Result<T, SentinelError>, error_code: u64) -> Json {
    match r {
        Ok(r) => create_json_rpc_response(id, r),
        Err(SentinelError::Json(ref j)) => create_json_rpc_error_with_json(id, error_code, j),
        Err(e) => create_json_rpc_error(id, error_code, &e.to_string()),
    }
}

pub(crate) enum RpcCalls {
    Ping(RpcId),
    Unknown(RpcId, String),
    SignMessage(RpcId, RpcParams),
    Get(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    Put(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    GetInclusionProof(RpcId, WebSocketTx, CoreCxnStatus),
    Delete(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    GetStatus(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    HardReset(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    GetCoreState(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    GetAttestionCertificate(RpcId, WebSocketTx, CoreCxnStatus),
    AddDebugSigners(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    StopSyncer(RpcId, BroadcastChannelTx, RpcParams, CoreCxnStatus),
    RemoveDebugSigner(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    StartSyncer(RpcId, BroadcastChannelTx, RpcParams, CoreCxnStatus),
    GetBalances(RpcId, Box<SentinelConfig>, RpcParams, EthRpcSenders),
    GetAttestionSignature(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    GetRegistrationSignature(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    LatestBlockInfos(RpcId, Box<SentinelConfig>, WebSocketTx, CoreCxnStatus),
    GetRegistrationExtensionTx(RpcId, Box<SentinelConfig>, RpcParams, EthRpcSenders),
    Init(
        RpcId,
        Box<SentinelConfig>,
        EthRpcSenders,
        WebSocketTx,
        RpcParams,
        CoreCxnStatus,
    ),
    GetSyncState(RpcId, Box<SentinelConfig>, WebSocketTx, EthRpcSenders, CoreCxnStatus),
    ResetChain(
        RpcId,
        Box<SentinelConfig>,
        EthRpcSenders,
        WebSocketTx,
        RpcParams,
        CoreCxnStatus,
    ),
    ProcessBlock(RpcId, Box<SentinelConfig>, EthRpcSenders, WebSocketTx, RpcParams, bool),
}

impl RpcCalls {
    pub(super) fn new(
        r: JsonRpcRequest,
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        eth_rpc_senders: EthRpcSenders,
        broadcast_channel_tx: BroadcastChannelTx,
        core_cxn: bool,
    ) -> Self {
        match r.method().as_ref() {
            "ping" => Self::Ping(*r.id()),
            "get" => Self::Get(*r.id(), websocket_tx, r.params(), core_cxn),
            "put" => Self::Put(*r.id(), websocket_tx, r.params(), core_cxn),
            "signMessage" | "sign" => Self::SignMessage(*r.id(), r.params()),
            "delete" => Self::Delete(*r.id(), websocket_tx, r.params(), core_cxn),
            "getInclusionProof" => Self::GetInclusionProof(*r.id(), websocket_tx, core_cxn),
            "hardReset" => Self::HardReset(*r.id(), r.params(), websocket_tx.clone(), core_cxn),
            "stopSyncer" => Self::StopSyncer(*r.id(), broadcast_channel_tx, r.params(), core_cxn),
            "getStatus" | "status" => Self::GetStatus(*r.id(), websocket_tx, r.params(), core_cxn),
            "startSyncer" => Self::StartSyncer(*r.id(), broadcast_channel_tx, r.params(), core_cxn),
            "getBalances" => Self::GetBalances(*r.id(), Box::new(config), r.params(), eth_rpc_senders),
            "removeDebugSigner" => Self::RemoveDebugSigner(*r.id(), r.params(), websocket_tx, core_cxn),
            "getAttestationCertificate" => Self::GetAttestionCertificate(*r.id(), websocket_tx, core_cxn),
            "getAttestationSignature" => Self::GetAttestionSignature(*r.id(), r.params(), websocket_tx, core_cxn),
            "addDebugSigners" | "addDebugSigner" => Self::AddDebugSigners(*r.id(), r.params(), websocket_tx, core_cxn),
            "getRegistrationExtensionTx" => {
                Self::GetRegistrationExtensionTx(*r.id(), Box::new(config.clone()), r.params(), eth_rpc_senders.clone())
            },
            "getRegistrationSignature" | "getRegSig" => {
                Self::GetRegistrationSignature(*r.id(), websocket_tx, r.params(), core_cxn)
            },
            "getLatestBlockInfos" | "latest" => {
                Self::LatestBlockInfos(*r.id(), Box::new(config.clone()), websocket_tx, core_cxn)
            },
            "getCoreState" | "getEnclaveState" | "state" => {
                Self::GetCoreState(*r.id(), r.params(), websocket_tx, core_cxn)
            },
            "getSyncState" => Self::GetSyncState(*r.id(), Box::new(config), websocket_tx, eth_rpc_senders, core_cxn),
            "reset" | "resetChain" => Self::ResetChain(
                *r.id(),
                Box::new(config),
                eth_rpc_senders,
                websocket_tx,
                r.params(),
                core_cxn,
            ),
            "init" => Self::Init(
                *r.id(),
                Box::new(config),
                eth_rpc_senders,
                websocket_tx,
                r.params(),
                core_cxn,
            ),
            "processBlock" | "process" | "submitBlock" | "submit" => Self::ProcessBlock(
                *r.id(),
                Box::new(config),
                eth_rpc_senders,
                websocket_tx,
                r.params(),
                core_cxn,
            ),
            _ => Self::Unknown(*r.id(), r.method()),
        }
    }

    pub(crate) fn check_core_is_connected(is_connected: bool) -> Result<(), SentinelError> {
        if is_connected {
            Ok(())
        } else {
            Err(SentinelError::NoCore)
        }
    }

    pub(crate) fn create_args(cmd: &str, params: RpcParams) -> RpcParams {
        [vec![cmd.to_string()], params].concat()
    }

    pub(crate) fn check_params(
        params: RpcParams,
        min_required_num_params: usize,
    ) -> Result<RpcParams, WebSocketMessagesError> {
        if params.len() < min_required_num_params {
            Err(WebSocketMessagesError::NotEnoughArgs {
                got: params.len(),
                expected: min_required_num_params,
                args: params,
            })
        } else {
            Ok(params)
        }
    }

    pub(super) async fn handle(self) -> Result<impl warp::Reply, Rejection> {
        match self {
            Self::GetSyncState(id, config, websocket_tx, eth_rpc_senders, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_sync_state(*config, websocket_tx, eth_rpc_senders, core_cxn).await,
            ),
            Self::AddDebugSigners(id, params, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_add_debug_signers(params, websocket_tx, core_cxn).await)
            },
            Self::RemoveDebugSigner(id, params, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_remove_debug_signer(params, websocket_tx, core_cxn).await,
            ),
            Self::HardReset(id, params, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_hard_reset(params, websocket_tx, core_cxn).await)
            },
            Self::GetRegistrationSignature(id, websocket_tx, params, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_registration_signature(websocket_tx, params, core_cxn).await,
            ),
            Self::GetRegistrationExtensionTx(id, config, params, eth_rpc_senders) => {
                let err_msg = "could not get private key from environment!";
                if let Err(e) = Env::init() {
                    error!("{e}");
                    return Ok(warp::reply::json(&create_json_rpc_error(id, 1337, err_msg)));
                };

                let pk = match Env::get_private_key() {
                    Ok(k) => k,
                    Err(e) => {
                        error!("{e}");
                        return Ok(warp::reply::json(&create_json_rpc_error(id, 1337, err_msg)));
                    },
                };

                let result = Self::handle_get_registration_extension_tx(*config, params, pk, eth_rpc_senders).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::Get(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get(websocket_tx, params, core_cxn).await)
            },
            Self::GetStatus(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_status(websocket_tx, params, core_cxn).await)
            },
            Self::SignMessage(id, params) => {
                let result = Self::handle_sign_message(params).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::Put(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_put(websocket_tx, params, core_cxn).await)
            },
            Self::Delete(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_delete(websocket_tx, params, core_cxn).await)
            },
            Self::ResetChain(id, config, eth_rpc_senders, websocket_tx, params, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_reset_chain(*config, eth_rpc_senders, websocket_tx, params, core_cxn).await,
            ),
            Self::StopSyncer(id, broadcast_channel_tx, params, core_cxn) => {
                let result = Self::handle_syncer_start_stop(broadcast_channel_tx, params, true, core_cxn).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::GetBalances(id, config, params, eth_rpc_senders) => {
                let result = Self::handle_get_balances(*config, params, eth_rpc_senders).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::StartSyncer(id, broadcast_channel_tx, params, core_cxn) => {
                // TODO enum for syncer state
                let result = Self::handle_syncer_start_stop(broadcast_channel_tx, params, false, core_cxn).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::LatestBlockInfos(id, config, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_latest_block_infos(*config, websocket_tx, core_cxn).await,
            ),
            Self::ProcessBlock(id, config, eth_rpc_senders, websocket_tx, params, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_process_block(*config, eth_rpc_senders, websocket_tx, params, core_cxn).await,
            ),
            Self::Ping(id) => Ok(warp::reply::json(&create_json_rpc_response(id, "pong"))),
            Self::Init(id, config, eth_rpc_senders, websocket_tx, params, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_init(*config, websocket_tx, eth_rpc_senders, params, core_cxn).await,
            ),
            Self::GetCoreState(id, params, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_core_state(params, websocket_tx, core_cxn).await)
            },
            Self::GetAttestionCertificate(id, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_attestation_certificate(websocket_tx, core_cxn).await,
            ),
            Self::GetAttestionSignature(id, params, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_attestation_signature(websocket_tx, params, core_cxn).await,
            ),
            Self::GetInclusionProof(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_inclusion_proof(websocket_tx, core_cxn).await)
            },
            Self::Unknown(id, method) => Ok(warp::reply::json(&create_json_rpc_error(
                id,
                1, // FIXME arbitrary
                &format!("unknown method: {method}"),
            ))),
        }
    }

    // NOTE: This is because anything involving the core returns an encodable result from which we
    // need to extract either the successful json response, or turn an error into an error response.
    fn handle_ws_result(
        id: RpcId,
        r: Result<WebSocketMessagesEncodable, SentinelError>,
    ) -> Result<warp::reply::Json, Rejection> {
        debug!("handling websocket encodable result: {r:?}");
        let error_code = 1337; // FIXME
        let j = match r {
            Ok(WebSocketMessagesEncodable::Success(j)) => create_json_rpc_response(id, j),
            Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::Json(ref j))) => {
                // NOTE: Otherwise we end up double stringifying the json
                create_json_rpc_error_with_json(id, error_code, j)
            },
            Ok(WebSocketMessagesEncodable::Error(e)) => {
                let s = e.to_string();
                // NOTE: We can't actually _get_ the exceptions from JNI on the core side of
                // things, we can only ask the JNI env to print them to console for us. So alas we
                // can't really do much. We can't even get a string to manually parse for common
                // errors etc. As such, this is literally the best we can do.
                let err_msg = if s.contains("Java exception was thrown") {
                    "a java exception was thrown - please see core logs for details".to_string()
                } else {
                    s
                };
                create_json_rpc_error(id, error_code, &err_msg)
            },
            other => create_json_rpc_response_from_result(id, other, error_code),
        };
        Ok(warp::reply::json(&j))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_rpc_error_messages_should_not_be_double_stringified() {
        let error_code = 666;
        let id = Some(1337);
        let j = json!({"some": "string"});
        let r = create_json_rpc_error(id, error_code, &j.to_string());
        let x = create_json_rpc_error_with_json(id, error_code, &j);
        assert!(r.to_string().contains("\\\""));
        assert!(!x.to_string().contains("\\\""));
    }
}
