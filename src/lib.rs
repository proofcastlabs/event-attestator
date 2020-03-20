#![feature(try_trait)]
#![recursion_limit="128"] // NOTE: For the format! macro in block parsing.
#![feature(exclusive_range_pattern)]

pub mod btc_on_eth;
pub mod btc_on_eos;

#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[cfg(test)] extern crate simple_logger;

// TODO Refactor the app since the import structure can no longer be flat.
// TODO ||, expose only the API we want.
// TODO Drop the conditional compilation.
