use std::result;
use crate::btc_on_eos::errors::AppError;

pub type Byte = u8;
pub type Bytes = Vec<Byte>;
pub type DataSensitivity = Option<u8>;
pub type Result<T> = result::Result<T, AppError>;
