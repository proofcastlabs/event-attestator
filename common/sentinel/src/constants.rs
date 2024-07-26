pub const HEX_RADIX: u32 = 16;
pub const MIN_FREQUENCY: u64 = 15;
pub const MAX_FREQUENCY: u64 = 60 * 10;
pub const DEFAULT_SLEEP_TIME: u64 = 15_000;
pub const MAX_CHANNEL_CAPACITY: usize = 1337;
pub const MILLISECONDS_MULTIPLIER: u64 = 1000;

lazy_static! {
    pub static ref HOST_PROTOCOL_ID: common_network_ids::ProtocolId = common_network_ids::ProtocolId::Ethereum;
    pub static ref NATIVE_PROTOCOL_ID: common_network_ids::ProtocolId = common_network_ids::ProtocolId::Ethereum;
}
