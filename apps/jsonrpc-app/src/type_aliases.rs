use common_sentinel::{BroadcastChannelMessages, EthRpcMessages, WebSocketMessages};
use tokio::sync::{
    broadcast::{Receiver as MpmcRx, Sender as MpmcTx},
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
};

pub(crate) type CoreCxnStatus = bool;
pub(crate) type EthRpcTx = MpscTx<EthRpcMessages>;
pub(crate) type WebSocketRx = MpscRx<WebSocketMessages>;
pub(crate) type WebSocketTx = MpscTx<WebSocketMessages>;
pub(crate) type BroadcastChannelTx = MpmcTx<BroadcastChannelMessages>;
pub(crate) type BroadcastChannelRx = MpmcRx<BroadcastChannelMessages>;
