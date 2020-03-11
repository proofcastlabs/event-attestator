use hex;
use log;
use std::fmt;
use serde_json;
use std::error::Error;

#[derive(Debug)]
pub enum AppError {
    Custom(String),
    IOError(std::io::Error),
    HexError(hex::FromHexError),
    CryptoError(secp256k1::Error),
    SerdeJsonError(serde_json::Error),
    NoneError(std::option::NoneError),
    FromUtf8Error(std::str::Utf8Error),
    SetLoggerError(log::SetLoggerError),
    ParseIntError(std::num::ParseIntError),
    ChronoError(chrono::format::ParseError),
    EosPrimitivesError(eos_primitives::Error),
    SystemTimeError(std::time::SystemTimeError),
    Base58Error(crate::btc_on_eos::base58::Error),
    FromSliceError(std::array::TryFromSliceError),
    EosPrimitivesNamesError(eos_primitives::ParseNameError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            AppError::Custom(ref msg) =>
                format!("{}", msg),
            AppError::HexError(ref e) =>
                format!("✘ Hex Error!\n✘ {}", e),
            AppError::IOError(ref e) =>
                format!("✘ I/O Error!\n✘ {}", e),
            AppError::CryptoError(ref e) =>
                format!("✘ Crypto Error!\n✘ {}", e),
            AppError::Base58Error(ref e) =>
                format!("✘ Base58 Error!\n✘ {}", e),
            AppError::ChronoError(ref e) =>
                format!("✘ Chrono error: \n✘ {:?}", e),
            AppError::SerdeJsonError(ref e) =>
                format!("✘ Serde-Json Error!\n✘ {}", e),
            AppError::ParseIntError(ref e) =>
                format!("✘ Parse Int Error!\n✘ {:?}", e),
            AppError::SystemTimeError(ref e) =>
                format!("✘ System Time Error!\n✘ {}", e),
            AppError::FromUtf8Error(ref e) =>
                format!("✘ From utf8 error: \n✘ {:?}", e),
            AppError::FromSliceError(ref e) =>
                format!("✘ From slice error: \n✘ {:?}", e),
            AppError::NoneError(ref e) =>
                format!("✘ Nothing to unwrap!\n✘ {:?}", e),
            AppError::EosPrimitivesError(ref e) =>
                format!("✘ Eos Primitives Error!\n✘ {:?}", e),
            AppError::SetLoggerError(ref e) =>
                format!("✘ Error setting up logger!\n✘ {}", e),
            AppError::EosPrimitivesNamesError(ref e) =>
                format!("✘ Eos Primitives Names Error!\n✘ {:?}", e),
        };
        f.write_fmt(format_args!("{}", msg))
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        "\n✘ Program Error!\n"
    }
}

impl From<hex::FromHexError> for AppError {
    fn from(e: hex::FromHexError) -> AppError {
        AppError::HexError(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> AppError {
        AppError::IOError(e)
    }
}

impl From<std::option::NoneError> for AppError {
    fn from(e: std::option::NoneError) -> AppError {
        AppError::NoneError(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> AppError {
        AppError::SerdeJsonError(e)
    }
}

impl From<log::SetLoggerError> for AppError {
    fn from(e: log::SetLoggerError) -> AppError {
        AppError::SetLoggerError(e)
    }
}

impl From<secp256k1::Error> for AppError {
    fn from(e: secp256k1::Error) -> AppError {
        AppError::CryptoError(e)
    }
}

impl From<std::time::SystemTimeError> for AppError {
    fn from(e: std::time::SystemTimeError) -> AppError {
        AppError::SystemTimeError(e)
    }
}

impl From<eos_primitives::Error> for AppError {
    fn from(e: eos_primitives::Error) -> AppError {
        AppError::EosPrimitivesError(e)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> AppError {
        AppError::ParseIntError(e)
    }
}

impl From<crate::btc_on_eos::base58::Error> for AppError {
    fn from(e: crate::btc_on_eos::base58::Error) -> AppError {
        AppError::Base58Error(e)
    }
}

impl From<eos_primitives::ParseNameError> for AppError {
    fn from(e: eos_primitives::ParseNameError) -> AppError {
        AppError::EosPrimitivesNamesError(e)
    }
}

impl From<std::array::TryFromSliceError> for AppError {
    fn from(e: std::array::TryFromSliceError) -> AppError {
        AppError::FromSliceError(e)
    }
}

impl From<std::str::Utf8Error> for AppError {
    fn from(e: std::str::Utf8Error) -> AppError {
        AppError::FromUtf8Error(e)
    }
}

impl From<chrono::format::ParseError> for AppError {
    fn from(e: chrono::format::ParseError) -> AppError {
        AppError::ChronoError(e)
    }
}
