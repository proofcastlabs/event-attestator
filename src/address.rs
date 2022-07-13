use std::{fmt, str::FromStr};

use bitcoin::util::address::Address as BtcAddress;
use eos_chain::AccountName as EosAddress;
use ethereum_types::Address as EthAddress;
use rust_algorand::AlgorandAddress as AlgoAddress;

use crate::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_hex_to_eth_address},
    errors::AppError,
    types::Result,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Address {
    Btc(String),
    Eth(String),
    Eos(String),
    Algo(String),
}

macro_rules! impl_conversion_fxns {
    ($symbol:expr, $convert_to_self_fxn:expr, $convert_from_str_fxn:expr) => {
        paste! {
            impl From<&[< $symbol:camel Address >]> for Address {
                fn from(a: &[< $symbol:camel Address>]) -> Self {
                    Self::[< $symbol:camel >]($convert_from_str_fxn(a))
                }
            }

            impl TryInto<[< $symbol:camel Address >]> for Address {
                type Error = AppError;

                fn try_into(self) -> Result<[< $symbol:camel Address >]> {
                    match self {
                        Self::[< $symbol:camel >](a) => Ok($convert_to_self_fxn(&a)?),
                        _ => Err(format!("Cannot convert {} into {} address!", self, $symbol).into())
                    }
                }
            }
        }
    };
}

impl Address {
    fn to_string<T: ToString>(s: T) -> String {
        s.to_string()
    }
}

impl_conversion_fxns!("BTC", BtcAddress::from_str, Self::to_string);
impl_conversion_fxns!("EOS", EosAddress::from_str, Self::to_string);
impl_conversion_fxns!("ALGO", AlgoAddress::from_str, Self::to_string);
impl_conversion_fxns!("ETH", convert_hex_to_eth_address, convert_eth_address_to_string);

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Eos(a) => write!(f, "{}", a),
            Self::Btc(a) => write!(f, "{}", a),
            Self::Eth(a) => write!(f, "{}", a),
            Self::Algo(a) => write!(f, "{}", a),
        }
    }
}
