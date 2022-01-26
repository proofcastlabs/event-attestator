use rust_algorand::AlgorandHash;

pub const ALGO_TAIL_LENGTH: u64 = 30;
pub const ALGO_CORE_IS_INITIALIZED_JSON: &str = "{algo_core_initialized:true}";

lazy_static! {
    pub static ref ALGO_PTOKEN_GENESIS_HASH: AlgorandHash =
        AlgorandHash::from_bytes(&vec![0u8; 32]).expect("This cannot fail");
}
