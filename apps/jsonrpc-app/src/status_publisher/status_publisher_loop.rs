use common_network_ids::NetworkId;
use common_sentinel::{
    call_core,
    check_ipfs_daemon_is_running,
    publish_status as publish_status_via_ipfs,
    BroadcastChannelMessages,
    SentinelConfig,
    SentinelError,
    SentinelStatus,
    StatusPublisherBroadcastChannelMessages,
    StatusPublisherMessages,
    WebSocketMessagesEncodable,
};
use tokio::{
    sync::{broadcast::Receiver as MpMcRx, mpsc::Sender as MpscTx},
    time::{sleep, Duration},
};

use crate::type_aliases::{BroadcastChannelTx, CoreCxnStatus, StatusPublisherRx, StatusPublisherTx, WebSocketTx};

async fn publish_status(
    config: &SentinelConfig,
    websocket_tx: WebSocketTx,
    core_timeout: &u64,
    network_ids: Vec<NetworkId>,
) -> Result<(), SentinelError> {
    let core_result = call_core(
        *core_timeout,
        websocket_tx.clone(),
        WebSocketMessagesEncodable::GetStatus(network_ids),
    )
    .await?;

    let status = SentinelStatus::try_from(core_result)?;

    // FIXME error handling here. EG what if the ipfs node hasn't got `--enable-pubsub-experiment`
    // enabled?? We should just got back to sleep on this loop and report the error so the user can
    // sort out their daemon.

    Ok(publish_status_via_ipfs(config.ipfs().ipfs_bin_path(), status)?)
}

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: MpMcRx<BroadcastChannelMessages>,
) -> Result<StatusPublisherBroadcastChannelMessages, SentinelError> {
    // NOTE: This loops continuously listening to the broadcasting channel, and only returns if we
    // receive a pertinent message. This way, other messages won't cause early returns in the main
    // tokios::select, so that the main_loop can continue doing its work.
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::StatusPublisher(msg)) => break 'broadcast_channel_loop Ok(msg),
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for us
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

async fn publish_status_loop(
    frequency: &u64,
    status_tx: MpscTx<StatusPublisherMessages>,
    core_cxn_status: &CoreCxnStatus,
    status_publisher_is_enabled: &bool,
    ipfs_bin_path: &str,
) -> Result<(), SentinelError> {
    // NOTE: This loop runs to send messages to the status loop at a configurable frequency to tell
    // it to publish its status. It should never return, except in error.
    'publish_status_loop: loop {
        info!("status publisher sleeping for {frequency}s...");
        sleep(Duration::from_secs(*frequency)).await;

        if !core_cxn_status {
            warn!("core is currently not connected so cannot publish a status update!");
            continue 'publish_status_loop;
        }

        if !status_publisher_is_enabled {
            warn!("status publisher currently disabled so will not publish a status update!");
            continue 'publish_status_loop;
        }

        if let Err(e) = check_ipfs_daemon_is_running(ipfs_bin_path) {
            error!("ipfs daemon issue, cannot publish a status update, please check your daemon");
            error!("{e}");
            continue 'publish_status_loop;
        }

        info!("{frequency}s has elapsed - sending message to publish status...");
        match status_tx.send(StatusPublisherMessages::SendStatusUpdate).await {
            Ok(_) => continue 'publish_status_loop,
            Err(e) => break 'publish_status_loop Err(e.into()),
        }
    }
}

pub async fn status_publisher_loop(
    config: SentinelConfig,
    mut status_rx: StatusPublisherRx,
    status_tx: StatusPublisherTx,
    broadcast_channel_tx: BroadcastChannelTx,
    websocket_tx: WebSocketTx,
    disable: bool,
) -> Result<(), SentinelError> {
    let name = "status publisher loop";
    let ipfs_bin_path = config.ipfs().ipfs_bin_path();
    let mut core_is_connected = false;
    let network_ids = config.network_ids();
    let mut status_publisher_is_enabled = !disable;

    if status_publisher_is_enabled {
        // NOTE: this will actually _error_ and exit the program if the check fails. Whereas later
        // checks in the loop will simply print an error to the console imploring the user to check
        // and start their IPFS daemon.
        check_ipfs_daemon_is_running(ipfs_bin_path)?;
    };

    let core_timeout = *config.core().timeout(); // TODO Make updateable via rpc call
    let mut status_update_frequency = *config.ipfs().status_update_frequency();

    'status_loop: loop {
        tokio::select! {
            r = publish_status_loop(&status_update_frequency, status_tx.clone(), &core_is_connected, &status_publisher_is_enabled, ipfs_bin_path) => {
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
            r = status_rx.recv() => match r {
                Some(StatusPublisherMessages::SendStatusUpdate) => {
                    if !core_is_connected {
                        warn!("not publishing status because no core is connected");
                        continue 'status_loop
                    } else {
                        match publish_status(
                            &config,
                            websocket_tx.clone(),
                            &core_timeout,
                            network_ids.clone(),
                        ).await {
                            Ok(_) => continue 'status_loop,
                            Err(e) => break 'status_loop Err(e)
                        }
                    }
                },
                Some(StatusPublisherMessages::SetStatusPublishingFreqency(new_frequency)) => {
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
                            StatusPublisherBroadcastChannelMessages::Stop => {
                                warn!("msg received to stop the publishing status updates {note}");
                                status_publisher_is_enabled = false;
                                continue 'status_loop
                            },
                            StatusPublisherBroadcastChannelMessages::Start => {
                                warn!("msg received to start publishing status updates {note}");
                                status_publisher_is_enabled = true;
                                continue 'status_loop
                            },
                            StatusPublisherBroadcastChannelMessages::CoreConnected => {
                                warn!("core connected message received in {name} {note}");
                                core_is_connected = true;
                                continue 'status_loop
                            },
                            StatusPublisherBroadcastChannelMessages::CoreDisconnected => {
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
                warn!("in {name} `else` branch, {name} is currently {}abled", if status_publisher_is_enabled { "en" } else { "dis" });
                continue 'status_loop
            },
        }
    }
}
