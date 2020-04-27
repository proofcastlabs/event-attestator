#![feature(try_trait)]
#![recursion_limit="128"] // NOTE: For the format! macro in block parsing.
#![feature(exclusive_range_pattern)]

pub mod utils;
pub mod types;
pub mod traits;
pub mod chains;
pub mod errors;
pub mod base58;
pub mod constants;
pub mod btc_on_eth;
pub mod btc_on_eos;
pub mod check_debug_mode;
pub mod debug_database_utils;

#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[cfg(test)] extern crate simple_logger;

// TODO Refactor the app since the import structure can no longer be flat.
// TODO ||, expose only the API we want.
// TODO Drop the conditional compilation.
