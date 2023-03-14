use std::result::Result;

use lib::{get_sub_mat, Batch, BroadcastMessages, EndpointError, ProcessorMessages, SentinelError, SyncerMessages};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time::{sleep, Duration},
};

// TODO split this up? How to share the code though :/ Macro + paste?
// TODO We need a channel to tell a syncer to restart from a given block number, in case of
// processing errors etc.

/*
 can we tokio select like:
_native_syncer_rx.recv {
    SyncerMessages::stop => {
        some indefinite timeout, or what??
        OR send a watch channel with the above message? Then we watch for the change before restarting the loop?
    },
    SyncerMessages::start => {
        start the native loop again?
    },
}
 */
async fn native_loop(
    log_prefix: &str,
    mut batch: Batch,
    processor_tx: Sender<ProcessorMessages>,
    _native_syncer_rx: Receiver<SyncerMessages>,
) -> Result<(), SentinelError> {
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();
    let mut block_num = 16778137;

    'native_loop: loop {
        let maybe_block = get_sub_mat(&ws_client, block_num).await;

        if let Ok(block) = maybe_block {
            batch.push(block);
            if batch.is_ready_to_submit() {
                info!("{log_prefix} Batch is ready to submit!");
                match processor_tx.send(ProcessorMessages::ProcessNative(batch.to_submission_material())) {
                    Ok(_) => (),
                    Err(e) => break 'native_loop Err(e.into()),
                };
                batch.drain();
                // TODO start block number is wrong!
                continue 'native_loop;
            } else {
                block_num += 1;
                continue 'native_loop;
            }
        } else if let Err(SentinelError::Endpoint(EndpointError::NoBlock(_))) = maybe_block {
            info!("{log_prefix} No next block yet - sleeping for {sleep_duration}ms...");
            sleep(Duration::from_millis(sleep_duration)).await;
            continue 'native_loop;
        } else if let Err(e) = maybe_block {
            return Err(e);
        }
    }
}

async fn host_loop(
    log_prefix: &str,
    mut batch: Batch,
    processor_tx: Sender<ProcessorMessages>,
    _host_syncer_rx: Receiver<SyncerMessages>,
) -> Result<(), SentinelError> {
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();
    let mut block_num = 16778137;

    'host_loop: loop {
        let maybe_block = get_sub_mat(&ws_client, block_num).await;

        if let Ok(block) = maybe_block {
            batch.push(block);
            if batch.is_ready_to_submit() {
                info!("{log_prefix} Batch is ready to submit!");
                match processor_tx.send(ProcessorMessages::ProcessHost(batch.to_submission_material())) {
                    Ok(_) => (),
                    Err(e) => break 'host_loop Err(e.into()),
                };
                batch.drain();
                // TODO start block number is wrong!
                continue 'host_loop;
            } else {
                block_num += 1;
                continue 'host_loop;
            }
        } else if let Err(SentinelError::Endpoint(EndpointError::NoBlock(_))) = maybe_block {
            info!("{log_prefix} No next block yet - sleeping for {sleep_duration}ms...");
            sleep(Duration::from_millis(sleep_duration)).await;
            continue 'host_loop;
        } else if let Err(e) = maybe_block {
            return Err(e);
        }
    }
}

pub async fn native_syncer_loop(
    batch: Batch,
    _broadcast_rx: Receiver<BroadcastMessages>,
    syncer_rx: Receiver<SyncerMessages>,
    processor_tx: Sender<ProcessorMessages>,
) -> Result<(), SentinelError> {
    let log_prefix = "native_syncer:".to_string();

    tokio::select! {
        res = native_loop(&log_prefix, batch, processor_tx, syncer_rx) => res,
        _ = tokio::signal::ctrl_c() => {
            warn!("{log_prefix} shutting down...");
            Err(SentinelError::SigInt("native syncer".into()))
        },
    }
}

pub async fn host_syncer_loop(
    batch: Batch,
    _broadcast_rx: Receiver<BroadcastMessages>,
    syncer_rx: Receiver<SyncerMessages>,
    processor_tx: Sender<ProcessorMessages>,
) -> Result<(), SentinelError> {
    let log_prefix = "host_syncer:";

    tokio::select! {
        res = host_loop(log_prefix, batch, processor_tx, syncer_rx) => res,
        _ = tokio::signal::ctrl_c() => {
            warn!("{log_prefix} shutting down...");
            Err(SentinelError::SigInt("host syncer".into()))
        },
    }
}
