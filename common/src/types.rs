//! # pToken types.
use std::result;

use crate::errors::AppError;
// NOTE: Temporary, until try_trait is stabilized
pub use crate::errors::AppError::NoneError;

pub type Bytes = Vec<Byte>;
pub type Result<T> = result::Result<T, AppError>;

pub type Byte = u8;
pub type DataSensitivity = Option<u8>;
