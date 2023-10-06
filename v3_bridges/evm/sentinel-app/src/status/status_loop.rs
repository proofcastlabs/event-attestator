use std::str::FromStr;

use common_metadata::MetadataChainId;
use common_sentinel::{
    check_ipfs_daemon_is_running,
    BroadcastChannelMessages,
    SentinelConfig,
    SentinelError,
    SentinelStatus,
    StatusBroadcastChannelMessages,
    StatusMessages,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use tokio::{
    sync::{
        broadcast::{Receiver as MpMcRx, Sender as MpMcTx},
        mpsc::{Receiver as MpscRx, Sender as MpscTx},
    },
    time::{sleep, Duration},
};

use crate::type_aliases::WebSocketTx;

async fn publish_status(config: &SentinelConfig, websocket_tx: MpscTx<WebSocketMessages>) -> Result<(), SentinelError> {
    // [ ] get status from core
    // [ ] publish it

    /*
    let mcids = params
        .iter()
        .map(|s| MetadataChainId::from_str(s).map_err(|_| WebSocketMessagesError::ParseMetadataChainId(s.into())))
        .collect::<Result<Vec<MetadataChainId>, WebSocketMessagesError>>()?;
        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetStatus(mcids));
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "getting status";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
    */

    todo!("this");
    Ok(())
}

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: MpMcRx<BroadcastChannelMessages>,
) -> Result<StatusBroadcastChannelMessages, SentinelError> {
    // NOTE: This loops continuously listening to the broadcasting channel, and only returns if we
    // receive a pertinent message. This way, other messages won't cause early returns in the main
    // tokios::select, so that the main_loop can continue doing its work.
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::Status(msg)) => break 'broadcast_channel_loop Ok(msg),
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for us
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

async fn publish_status_loop(frequency: u64, status_tx: MpscTx<StatusMessages>) -> Result<(), SentinelError> {
    // NOTE: This loop runs to send messages to the status loop at a configurable frequency to tell
    // it to publish its status. It should never return, except in error.

    todo!("take core cxn so we can only run the fxn to publish the status if the core is connected (below will have to turn into i)");

    'publish_status_loop: loop {
        sleep(Duration::from_secs(frequency)).await;
        warn!("{frequency}s has elapsed - sending message to cancel any cancellable user ops");
        /*
        match broadcaster_tx.send(BroadcasterMessages::CancelUserOps).await {
            Ok(_) => continue 'publish_status_loop,
            Err(e) => break 'publish_status_loop Err(e.into()),
        }
        */
    }
}

pub async fn status_loop(
    config: SentinelConfig,
    mut status_rx: MpscRx<StatusMessages>,
    status_tx: MpscTx<StatusMessages>,
    broadcast_channel_tx: MpMcTx<BroadcastChannelMessages>,
    websocket_tx: MpscTx<WebSocketMessages>,
) -> Result<(), SentinelError> {
    let name = "status loop";

    check_ipfs_daemon_is_running(config.ipfs().ipfs_bin_path())?;

    let mut core_is_connected = false;
    let mut status_is_enabled = false;
    let mut status_update_frequency = *config.ipfs().status_update_frequency();

    'status_loop: loop {
        tokio::select! {
            r = publish_status_loop(status_update_frequency, status_tx.clone()) => {
                match r {
                    Ok(_) => {
                        warn!("publish status loop returned Ok(()) for some reason");
                    },
                    Err(e) => {
                        error!("publish status publisher loop error: {e}");
                    }
                }

                let sleep_time = 30; // FIXME make configurable
                warn!("sleeping for {sleep_time}s and restarting status loop");
                sleep(Duration::from_secs(sleep_time)).await;
                continue 'status_loop
            },
            r = status_rx.recv() , if status_is_enabled && core_is_connected => match r {
                Some(StatusMessages::SendStatusUpdate) => {




                    todo!("this"); // TODO TODO TODO TODO





                    continue 'status_loop
                },
                Some(StatusMessages::SetStatusPublishingFreqency(new_frequency)) => {
                    status_update_frequency = new_frequency;
                    info!("updated publishing frequency to {new_frequency}");
                    continue 'status_loop
                },
                None => {
                    let m = "all {name} senders dropped!";
                    warn!("{m}");
                    break 'status_loop Err(SentinelError::Custom(name.into()))
                },
            },
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe()) => {
                match r {
                    Err(e) => break 'status_loop Err(e),
                    Ok(msg) => {
                        let note = format!("(core is currently {}connected)", if core_is_connected { "" } else { "not "});
                        match msg {
                            StatusBroadcastChannelMessages::Stop => {
                                warn!("msg received to stop the {name} {note}");
                                status_is_enabled = false;
                                continue 'status_loop
                            },
                            StatusBroadcastChannelMessages::Start => {
                                warn!("msg received to start the {name} {note}");
                                status_is_enabled = true;
                                continue 'status_loop
                            },
                            StatusBroadcastChannelMessages::CoreConnected => {
                                warn!("core connected message received in {name} {note}");
                                core_is_connected = true;
                                continue 'status_loop
                            },
                            StatusBroadcastChannelMessages::CoreDisconnected => {
                                warn!("core disconnected message received in {name} {note}");
                                core_is_connected = false;
                                continue 'status_loop
                            },
                        }
                    },
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'status_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if status_is_enabled { "en" } else { "dis" });
                continue 'status_loop
            },
        }
    }
}
