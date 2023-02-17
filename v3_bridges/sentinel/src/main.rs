mod get_block;
mod get_rpc_client;

use get_block::get_block;
use get_rpc_client::get_rpc_client;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    use simple_logger; // FIXME rm!
    simple_logger::init().unwrap(); // FIXME rm!

    let ws_client = get_rpc_client(url).await.unwrap();

    for i in 0..3 {
        get_block(&ws_client, 16640611 + i).await.unwrap();
    }
}
// TODO use https://crates.io/crates/async-log for async logging when we have > 1 thread

// TODO use confy for easy config management, and toml files - better than json here.
// Write logs to a file, and watch that during running. (see vanilla apps for how that's done)
// pull the vanilla apps into this workspace too to share deps (rocksdb mostly)
// JSON-RPC spec:https://www.jsonrpc.org/specificationhttps://www.jsonrpc.org/specification https://www.jsonrpc.org/specification
