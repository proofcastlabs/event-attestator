#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;

mod errors;
#[cfg(feature = "file-logger")]
mod file_logger;

#[cfg(feature = "std-err-logger")]
mod std_err_logger;

pub use self::errors::LoggerError;
#[cfg(feature = "file-logger")]
pub use self::file_logger::initialize_file_logger as init_logger;
#[cfg(feature = "std-err-logger")]
pub use self::std_err_logger::initialize_std_err_logger as init_logger;
