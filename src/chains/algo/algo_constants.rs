use rust_algorand::AlgorandHash;

#[cfg(test)]
pub const ALGO_ADDRESS_LENGTH_IN_BYTES: usize = 32;

pub const ALGO_TAIL_LENGTH: u64 = 30;
pub const MAX_BYTES_FOR_ALGO_USER_DATA: usize = 1000;
pub const ALGO_CORE_IS_INITIALIZED_JSON: &str = "{algo_core_initialized:true}";
pub const ALGO_SAFE_ADDRESS: &str = "GSKWPLI7YL7OF23F5ET5L7HSFLLJL3F5DUO7AH2HQLOSO4DRRHR76TDQ2I";

lazy_static! {
    pub static ref ALGO_PTOKEN_GENESIS_HASH: AlgorandHash = AlgorandHash::default();
}
