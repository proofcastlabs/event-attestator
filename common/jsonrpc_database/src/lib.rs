#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;

mod errors;
mod types;

mod json_rpc_database;

use crate::json_rpc_database::Database;

pub fn get_db() -> Result<Database, JsonRpcDatabaseError> {
    Database::open()
}

pub use crate::errors::JsonRpcDatabaseError;
