#![feature(try_trait)]
#![recursion_limit="128"]
#![feature(exclusive_range_pattern)]

pub use types::{
    Bytes,
    Result
};
pub use errors::AppError;
pub use traits::DatabaseInterface;

pub mod types;
pub mod traits;
pub mod errors;
pub mod btc_on_eth;
pub mod btc_on_eos;

mod utils;
mod base58;
mod chains;
mod constants;
mod check_debug_mode;
mod debug_database_utils;

#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[cfg(test)] extern crate simple_logger;
