use thiserror::Error;

#[derive(Debug, Error)]
pub enum BitcoinError {
    #[error("btc fee hard cap of {0} exceeded")]
    FeeHardCapExceeded(u64),
}
