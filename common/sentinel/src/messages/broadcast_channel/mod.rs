mod rpc_server;
mod syncer;

use common_network_ids::NetworkId;

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    RpcServer(RpcServerBroadcastChannelMessages),
    Syncer(NetworkId, SyncerBroadcastChannelMessages),
}

pub use self::{rpc_server::RpcServerBroadcastChannelMessages, syncer::SyncerBroadcastChannelMessages};
