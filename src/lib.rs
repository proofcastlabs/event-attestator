#![feature(try_trait)]
#![recursion_limit="128"] // NOTE: For the format! macro in block parsing.
#![feature(exclusive_range_pattern)]

pub mod btc_on_eth;
pub mod btc_on_eos;

mod utils;
mod types;
mod traits;
mod errors;
mod base58;
mod chains;
mod constants;
mod check_debug_mode;
mod debug_database_utils;

#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[cfg(test)] extern crate simple_logger;
