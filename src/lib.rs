#![feature(try_trait)]
#![recursion_limit="128"]
#![feature(exclusive_range_pattern)]

//! # The __`pToken`__ Core
//!
//! Herein lies the functionality required for the cross-chain conversions
//! between various blockchains allowing for decentalized swaps between a native
//! asset and a host chain's pTokenized version of that asset.
//!
//! __Note:__ When compiling the core, your app must select the desired pToken
//! via the __`Cargo.toml`__ like so:
//!
//! ```no_compile
//! ptokens_core = { version = "1.0.0", features = ["pbtc-on-eos"] }
//! ```
//!
//! ### __CAUTION:__
//!
//!  - Attempting to select multiple pTokens result in errors upon
//! compilaton due to multiple declarations of the same database key. This is
//! correct behaviour. Please select only one pToken type in your
//! __`Cargo.toml`__.
//!
//!  - If __`legacy`__ is selected, all constants are NOT prefixed at compile
//! time, meaning database key conflicts can occur. Use at your own risk.
//!

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
