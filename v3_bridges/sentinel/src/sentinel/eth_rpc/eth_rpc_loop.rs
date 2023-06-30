use std::result::Result;

use lib::{
    eth_call,
    get_eth_balance,
    get_gas_price,
    get_latest_block_num,
    get_nonce,
    get_sub_mat,
    get_user_op_state,
    push_tx,
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
};
use tokio::{
    sync::mpsc::Receiver as MpscRx,
    time::{sleep, Duration},
};

// NOTE: The underlying RPC calls have both retry & timeout logic, however in the event of a websocket disconnect, they
// immediately return with an error. That error is handled in each of the arms below, via rotating the endpoint to get a
// new socket.

// TODO DRY out the repeat code below, though it's not trivial due to having to replace the mutable websocket clients
// upon endpoint rotation.

const ENDPOINT_ROTATION_SLEEP_TIME: u64 = 2000;

pub async fn eth_rpc_loop(mut eth_rpc_rx: MpscRx<EthRpcMessages>, config: SentinelConfig) -> Result<(), SentinelError> {
    let mut h_endpoints = config.get_host_endpoints();
    let mut n_endpoints = config.get_native_endpoints();
    let n_sleep_time = n_endpoints.sleep_time();
    let h_sleep_time = h_endpoints.sleep_time();
    let mut h_ws_client = h_endpoints.get_first_ws_client().await?;
    let mut n_ws_client = n_endpoints.get_first_ws_client().await?;

    'eth_rpc_loop: loop {
        tokio::select! {
            r = eth_rpc_rx.recv() => match r {
                Some(msg) => {
                    match msg {
                        EthRpcMessages::GetUserOpState((side, user_op, contract_address, responder)) => {
                            'inner: loop {
                                let r = get_user_op_state(
                                    &user_op,
                                    &contract_address,
                                    if side.is_native() { &n_ws_client } else { &h_ws_client },
                                    if side.is_native() { n_sleep_time } else { h_sleep_time },
                                    side,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{side} eth rpc error: {e}");
                                        warn!("rotating {side} endpoint");
                                        sleep(Duration::from_millis(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        if side.is_native() {
                                            n_ws_client = n_endpoints.rotate().await?;
                                        } else {
                                            h_ws_client = h_endpoints.rotate().await?;
                                        };
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetLatestBlockNum((side, responder)) => {
                            'inner: loop {
                                let r = get_latest_block_num(
                                    if side.is_native() { &n_ws_client } else { &h_ws_client },
                                    if side.is_native() { n_sleep_time } else { h_sleep_time },
                                    side,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{side} eth rpc error: {e}");
                                        warn!("rotating {side} endpoint");
                                        sleep(Duration::from_millis(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        if side.is_native() {
                                            n_ws_client = n_endpoints.rotate().await?;
                                        } else {
                                            h_ws_client = h_endpoints.rotate().await?;
                                        };
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetGasPrice((side, responder)) => {
                            'inner: loop {
                                let r = get_gas_price(
                                    if side.is_native() { &n_ws_client } else { &h_ws_client },
                                    if side.is_native() { n_sleep_time } else { h_sleep_time },
                                    side,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{side} eth rpc error: {e}");
                                        warn!("rotating {side} endpoint");
                                        sleep(Duration::from_millis(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        if side.is_native() {
                                            n_ws_client = n_endpoints.rotate().await?;
                                        } else {
                                            h_ws_client = h_endpoints.rotate().await?;
                                        };
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::PushTx((tx, side, responder)) => {
                            'inner: loop {
                                let r = push_tx(
                                    &tx,
                                    if side.is_native() { &n_ws_client } else { &h_ws_client },
                                    if side.is_native() { n_sleep_time } else { h_sleep_time },
                                    side,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{side} eth rpc error: {e}");
                                        warn!("rotating {side} endpoint");
                                        sleep(Duration::from_millis(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        if side.is_native() {
                                            n_ws_client = n_endpoints.rotate().await?;
                                        } else {
                                            h_ws_client = h_endpoints.rotate().await?;
                                        };
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetNonce((side, address, responder)) => {
                            'inner: loop {
                                let r = get_nonce(
                                    if side.is_native() { &n_ws_client } else { &h_ws_client },
                                    &address,
                                    if side.is_native() { n_sleep_time } else { h_sleep_time },
                                    side,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{side} eth rpc error: {e}");
                                        warn!("rotating {side} endpoint");
                                        sleep(Duration::from_millis(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        if side.is_native() {
                                            n_ws_client = n_endpoints.rotate().await?;
                                        } else {
                                            h_ws_client = h_endpoints.rotate().await?;
                                        };
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::EthCall((data, side, address, default_block_parameter, responder)) => {
                            'inner: loop {
                                let r = eth_call(
                                    &address,
                                    &data,
                                    &default_block_parameter,
                                    if side.is_native() { &n_ws_client } else { &h_ws_client },
                                    if side.is_native() { n_sleep_time } else { h_sleep_time },
                                    side,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{side} eth rpc error: {e}");
                                        warn!("rotating {side} endpoint");
                                        sleep(Duration::from_millis(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        if side.is_native() {
                                            n_ws_client = n_endpoints.rotate().await?;
                                        } else {
                                            h_ws_client = h_endpoints.rotate().await?;
                                        };
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetSubMat((side, block_num, responder)) => {
                            'inner: loop {
                                let r = get_sub_mat(
                                    if side.is_native() { &n_ws_client } else { &h_ws_client },
                                    block_num,
                                    if side.is_native() { n_sleep_time } else { h_sleep_time },
                                    side,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    }
                                    Err(e) => {
                                        error!("{side} eth rpc error: {e}");
                                        warn!("rotating {side} endpoint");
                                        sleep(Duration::from_millis(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        if side.is_native() {
                                            n_ws_client = n_endpoints.rotate().await?;
                                        } else {
                                            h_ws_client = h_endpoints.rotate().await?;
                                        };
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetEthBalance((side, address, responder)) => {
                            'inner: loop {
                                let r = get_eth_balance(
                                    if side.is_native() { &n_ws_client } else { &h_ws_client },
                                    &address,
                                    if side.is_native() { n_sleep_time } else { h_sleep_time },
                                    side,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    }
                                    Err(e) => {
                                        error!("{side} eth rpc error: {e}");
                                        warn!("rotating {side} endpoint");
                                        sleep(Duration::from_millis(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        if side.is_native() {
                                            n_ws_client = n_endpoints.rotate().await?;
                                        } else {
                                            h_ws_client = h_endpoints.rotate().await?;
                                        };
                                        continue 'inner
                                    },
                                }
                            }
                        },
                    }
                },
                None => {
                    let m = "all eth rpc senders dropped!";
                    error!("{m}");
                    break 'eth_rpc_loop Err(SentinelError::Custom(m.into()))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("eth rpc shutting down...");
                break 'eth_rpc_loop Err(SentinelError::SigInt("eth rpc".into()))
            },
        }
    }
}
