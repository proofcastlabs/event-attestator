mod rpc_server;
mod syncer;
mod user_op_canceller;

use common_network_ids::NetworkId;

#[derive(Debug, Clone)]
pub enum BroadcastChannelMessages {
    RpcServer(RpcServerBroadcastChannelMessages),
    Syncer(NetworkId, SyncerBroadcastChannelMessages),
    UserOpCanceller(UserOpCancellerBroadcastChannelMessages),
}

pub use self::{
    rpc_server::RpcServerBroadcastChannelMessages,
    syncer::SyncerBroadcastChannelMessages,
    user_op_canceller::UserOpCancellerBroadcastChannelMessages,
};
