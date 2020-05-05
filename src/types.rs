use std::result;
use crate::errors::AppError;

pub type Bytes = Vec<Byte>;
pub type Result<T> = result::Result<T, AppError>;

pub(crate) type Byte = u8;
pub(crate) type DataSensitivity = Option<u8>;
